# Mentor — GitHub sync setup

This lets anyone click **Connect GitHub** in Mentor and commit their tool files
to a repo of their own. It needs two things you set up **once**: a GitHub App
and this Cloudflare Worker (the OAuth broker). End users host nothing.

## 1. Register the GitHub App

<https://github.com/settings/apps> → **New GitHub App**

- **Name**: e.g. `Mentor Sync`
- **Homepage URL**: `https://maebli.github.io/mentor/`
- **Callback URL**: `https://<your-worker-subdomain>.workers.dev/callback`
  (you'll get the exact Worker URL in step 2 — you can edit this after.)
- **Request user authorization (OAuth) during installation**: on
- **Webhook**: uncheck **Active** (not needed)
- **Permissions → Repository → Contents**: **Read and write**
- **Where can this app be installed?**: **Any account** (so the public can use it)

Create it, then note the **Client ID**, and click **Generate a new client secret**.

## 2. Deploy the Worker

```sh
cd worker
npm i -g wrangler          # if you don't have it
wrangler login
wrangler secret put GITHUB_CLIENT_ID       # paste the App's Client ID
wrangler secret put GITHUB_CLIENT_SECRET   # paste the generated secret
wrangler deploy
```

`wrangler deploy` prints the Worker URL (e.g.
`https://mentor-github-oauth.<you>.workers.dev`). Put that same URL's `/callback`
into the GitHub App's Callback URL from step 1.

If your app isn't served from `https://maebli.github.io`, edit `ALLOWED_ORIGIN`
in `wrangler.toml` and redeploy.

## 3. Give these two values back to the app

The Mentor front end needs, in its config:

- **GitHub App Client ID** (from step 1)
- **Worker base URL** (from step 2, without `/callback`)

Once those are set, the in-app flow is: **Connect GitHub** opens a popup to
`<worker>/auth` → user authorizes → the Worker posts the token back → the app
lets them pick or create a repo and commits each tool's text via the GitHub
Contents/Git Data API (all from the browser, since `api.github.com` allows CORS).
