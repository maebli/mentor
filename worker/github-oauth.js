/**
 * Mentor — GitHub OAuth broker (Cloudflare Worker).
 *
 * The only non-static piece of Mentor. It holds the GitHub App's client secret
 * and performs the user-to-server OAuth code exchange that a browser can't do
 * itself (GitHub's token endpoint has no CORS). Every visitor shares this one
 * Worker; nobody else hosts anything.
 *
 * Flow (Decap/Netlify-CMS style):
 *   app popup -> GET /auth        -> 302 to github.com/login/oauth/authorize
 *   github    -> GET /callback    -> exchange code for a user token
 *                                 -> return an HTML page that postMessages the
 *                                    token back to the opener window, then closes
 *
 * Required secrets / vars (set with `wrangler secret put` / [vars]):
 *   GITHUB_CLIENT_ID      - the GitHub App's Client ID
 *   GITHUB_CLIENT_SECRET  - a generated client secret (SECRET — never in the app)
 *   ALLOWED_ORIGIN        - exact app origin allowed to receive the token,
 *                           e.g. https://maebli.github.io
 */

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const redirectUri = `${url.origin}/callback`;

    if (url.pathname === "/auth") {
      const state = crypto.randomUUID();
      const authorize = new URL("https://github.com/login/oauth/authorize");
      authorize.searchParams.set("client_id", env.GITHUB_CLIENT_ID);
      authorize.searchParams.set("redirect_uri", redirectUri);
      authorize.searchParams.set("state", state);
      // GitHub Apps derive scope from their configured permissions, so no
      // `scope` param is needed. Set the state cookie for CSRF protection.
      return new Response(null, {
        status: 302,
        headers: {
          Location: authorize.toString(),
          "Set-Cookie": `mentor_oauth_state=${state}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=600`,
        },
      });
    }

    if (url.pathname === "/callback") {
      const code = url.searchParams.get("code");
      const state = url.searchParams.get("state");
      const cookie = request.headers.get("Cookie") || "";
      const expected = /mentor_oauth_state=([^;]+)/.exec(cookie)?.[1];
      if (!code || !state || state !== expected) {
        return htmlMessage(env.ALLOWED_ORIGIN, { error: "bad_state" });
      }

      const tokenRes = await fetch("https://github.com/login/oauth/access_token", {
        method: "POST",
        headers: { "Content-Type": "application/json", Accept: "application/json" },
        body: JSON.stringify({
          client_id: env.GITHUB_CLIENT_ID,
          client_secret: env.GITHUB_CLIENT_SECRET,
          code,
          redirect_uri: redirectUri,
        }),
      });
      const data = await tokenRes.json();
      if (!data.access_token) {
        return htmlMessage(env.ALLOWED_ORIGIN, { error: data.error || "no_token" });
      }
      return htmlMessage(env.ALLOWED_ORIGIN, { token: data.access_token });
    }

    return new Response("Mentor OAuth broker", { status: 200 });
  },
};

/**
 * Return a tiny HTML page that hands `payload` to the opener window via
 * postMessage (restricted to ALLOWED_ORIGIN) and closes itself.
 */
function htmlMessage(allowedOrigin, payload) {
  const body = `<!doctype html><meta charset="utf-8"><title>Connecting…</title>
<script>
(function () {
  var payload = ${JSON.stringify(payload)};
  if (window.opener) {
    window.opener.postMessage({ source: "mentor-github-oauth", payload: payload }, ${JSON.stringify(
      allowedOrigin,
    )});
  }
  document.body.textContent = payload.token ? "Connected. You can close this window." : "Sign-in failed: " + payload.error;
  setTimeout(function () { window.close(); }, 400);
})();
</script>`;
  return new Response(body, {
    status: 200,
    headers: { "Content-Type": "text/html; charset=utf-8" },
  });
}
