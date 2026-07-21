/* Mentor — client-side export helpers (no external libraries).
 *
 * Rasterises a DOM node to PNG via the SVG <foreignObject> technique: clone the
 * node, embed the page stylesheet + resolved theme variables, wrap it in an SVG,
 * load that as an image and paint it onto a canvas. Everything stays offline and
 * same-origin, so the canvas is never tainted and can be copied or downloaded.
 */
(function () {
  "use strict";

  // Discover every custom property declared on :root, so we can inline their
  // resolved values onto the export wrapper (`:root { --x }` won't match it).
  // Works for any page — the app and the CRC board have different palettes.
  function rootVarNames() {
    var names = {};
    for (var i = 0; i < document.styleSheets.length; i++) {
      var rules;
      try { rules = document.styleSheets[i].cssRules; } catch (e) { continue; }
      if (!rules) continue;
      for (var j = 0; j < rules.length; j++) {
        var r = rules[j];
        if (!r.selectorText || !r.style) continue;
        var sels = r.selectorText.split(",");
        var hit = false;
        for (var k = 0; k < sels.length; k++) {
          if (sels[k].trim() === ":root") { hit = true; break; }
        }
        if (!hit) continue;
        for (var m = 0; m < r.style.length; m++) {
          if (r.style[m].indexOf("--") === 0) names[r.style[m]] = true;
        }
      }
    }
    return Object.keys(names);
  }

  function collectCss() {
    var out = "";
    for (var i = 0; i < document.styleSheets.length; i++) {
      var sheet = document.styleSheets[i];
      var rules;
      try { rules = sheet.cssRules; } catch (e) { continue; }
      if (!rules) continue;
      for (var j = 0; j < rules.length; j++) out += rules[j].cssText + "\n";
    }
    return out;
  }

  function nodeToBlob(node, opts) {
    opts = opts || {};
    var scale = opts.scale || 2;
    var pad = opts.pad != null ? opts.pad : 24;
    var rect = node.getBoundingClientRect();
    // Use scroll size so content that overflows a scrolling pane isn't clipped.
    var w = Math.max(1, Math.ceil(rect.width), node.scrollWidth) + pad * 2;
    var h = Math.max(1, Math.ceil(rect.height), node.scrollHeight) + pad * 2;

    var root = getComputedStyle(document.documentElement);
    var body = getComputedStyle(document.body);
    var bg = opts.bg || body.backgroundColor || "#15120d";
    // Theme variables (font stacks contain quotes) go into the CDATA stylesheet,
    // NOT an inline style attribute — quotes there would terminate it and corrupt
    // the XML, making the <img> decoder reject the whole SVG.
    var vars = rootVarNames().map(function (v) {
      return v + ":" + root.getPropertyValue(v);
    }).join(";");
    var rootRule =
      ".mentor-export-root{" + vars + ";background:" + bg +
      ";color:var(--text);font-family:var(--sans);box-sizing:border-box}";

    var clone = node.cloneNode(true);
    clone.style.overflow = "visible";
    clone.style.margin = "0";
    var serialized = new XMLSerializer().serializeToString(clone);

    var svg =
      '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h + '">' +
      '<foreignObject x="0" y="0" width="' + w + '" height="' + h + '">' +
      '<div xmlns="http://www.w3.org/1999/xhtml" class="mentor-export-root" style="width:' +
      w + "px;height:" + h + "px;padding:" + pad + 'px">' +
      "<style><![CDATA[" + rootRule + collectCss() + "]]></style>" +
      serialized +
      "</div></foreignObject></svg>";

    var url = "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg);

    return new Promise(function (resolve, reject) {
      var img = new Image();
      img.onload = function () {
        var canvas = document.createElement("canvas");
        canvas.width = w * scale;
        canvas.height = h * scale;
        var ctx = canvas.getContext("2d");
        ctx.setTransform(scale, 0, 0, scale, 0, 0);
        ctx.drawImage(img, 0, 0);
        canvas.toBlob(function (blob) { resolve(blob); }, "image/png");
      };
      img.onerror = reject;
      img.src = url;
    });
  }

  function saveBlob(blob, filename) {
    var a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(function () { URL.revokeObjectURL(a.href); }, 1000);
  }

  function flash(msg) {
    // Best-effort toast; tools without one just get nothing.
    var t = document.getElementById("mentor-toast");
    if (!t) {
      t = document.createElement("div");
      t.id = "mentor-toast";
      t.style.cssText =
        "position:fixed;bottom:20px;left:50%;transform:translateX(-50%);" +
        "background:#1c1811;color:#ece3d2;border:1px solid #322b20;border-radius:8px;" +
        "padding:8px 14px;font:13px system-ui,sans-serif;z-index:9999;opacity:0;" +
        "transition:opacity .15s;pointer-events:none";
      document.body.appendChild(t);
    }
    t.textContent = msg;
    t.style.opacity = "1";
    clearTimeout(t._timer);
    t._timer = setTimeout(function () { t.style.opacity = "0"; }, 1800);
  }

  window.MentorExport = {
    downloadPng: function (sel, filename) {
      var node = document.querySelector(sel);
      if (!node) return;
      nodeToBlob(node).then(function (blob) {
        if (blob) { saveBlob(blob, filename); flash("Saved " + filename); }
      }).catch(function () { flash("Could not render PNG"); });
    },
    copyPng: function (sel) {
      var node = document.querySelector(sel);
      if (!node) return;
      nodeToBlob(node).then(function (blob) {
        if (!blob) throw 0;
        if (!window.ClipboardItem || !navigator.clipboard || !navigator.clipboard.write) {
          throw 0;
        }
        var item = new ClipboardItem({ "image/png": blob });
        return navigator.clipboard.write([item]);
      }).then(function () { flash("PNG copied to clipboard"); })
        .catch(function () { flash("Clipboard blocked — use Download PNG"); });
    },
    downloadText: function (text, filename) {
      saveBlob(new Blob([text], { type: "text/plain;charset=utf-8" }), filename);
      flash("Saved " + filename);
    },
    copyText: function (text) {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(
          function () { flash("Text copied to clipboard"); },
          function () { flash("Clipboard blocked"); }
        );
      } else {
        flash("Clipboard unavailable");
      }
    },
    // Exposed for the CRC board iframe, which rasterises its own node.
    _nodeToBlob: nodeToBlob,
    _saveBlob: saveBlob,
    _flash: flash,
  };
})();
