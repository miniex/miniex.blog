// Graph & Chart renderer — pastel theme matched
// Lazy-loads function-plot / Chart.js only when targets exist.

(function () {
  "use strict";

  // i18n strings (populated from #i18n-data if available)
  var _i18nEl = document.getElementById("i18n-data");
  var _i18n = _i18nEl ? JSON.parse(_i18nEl.textContent) : {};

  // ── Pastel palettes (light / dark) ──

  var PALETTE_LIGHT = {
    text: "#1a1a1a",
    textMuted: "rgba(0,0,0,0.7)",
    grid: "rgba(0,0,0,0.12)",
    gridDash: "3,3",
    border: "rgba(0,0,0,0.2)",
    tooltipBg: "rgba(252,250,251,0.96)",
    series: [
      "#c9899e",
      "#b096b8",
      "#d4a89e",
      "#8db8a0",
      "#d4c098",
      "#c98a8a",
      "#a0b8c8",
      "#b8a090",
    ],
  };

  var PALETTE_DARK = {
    text: "#e0d8dc",
    textMuted: "rgba(224,216,220,0.65)",
    grid: "rgba(208,160,176,0.12)",
    gridDash: "3,3",
    border: "rgba(208,160,176,0.2)",
    tooltipBg: "rgba(28,24,26,0.94)",
    series: [
      "#d0a0b0",
      "#baa0c0",
      "#d4b0a8",
      "#a0c8b0",
      "#d4c8a8",
      "#d0a0a0",
      "#a8c0d0",
      "#c8b0a0",
    ],
  };

  function isMobile() {
    return window.innerWidth < 768;
  }

  function isDark() {
    return (
      (document.documentElement.getAttribute("data-theme") || "")
        .toLowerCase()
        .indexOf("dark") !== -1
    );
  }

  function pal() {
    return isDark() ? PALETTE_DARK : PALETTE_LIGHT;
  }

  // ── DSL parser ──

  function parseDSL(text) {
    var c = {};
    text.split("\n").forEach(function (line) {
      line = line.trim();
      if (!line) return;
      var idx = line.indexOf(":");
      if (idx === -1) return;
      var k = line.substring(0, idx).trim().toLowerCase();
      var v = line.substring(idx + 1).trim();
      if (k === "fn" || k === "dataset") {
        (c[k] = c[k] || []).push(v);
      } else {
        c[k] = v;
      }
    });
    return c;
  }

  function parseRange(s) {
    var p = s.split(",").map(function (v) {
      return parseFloat(v.trim());
    });
    return p.length === 2 ? p : [-10, 10];
  }

  function pickColor(user, i, p) {
    return user || p.series[i % p.series.length];
  }

  function toRgba(color, a) {
    var N = {
      steelblue: [70, 130, 180],
      tomato: [255, 99, 71],
      green: [0, 128, 0],
      red: [255, 0, 0],
      blue: [0, 0, 255],
      orange: [255, 165, 0],
      purple: [128, 0, 128],
      gold: [255, 215, 0],
      teal: [0, 128, 128],
      coral: [255, 127, 80],
      crimson: [220, 20, 60],
      dodgerblue: [30, 144, 255],
      mediumseagreen: [60, 179, 113],
    };
    var lo = color.toLowerCase();
    if (N[lo]) return "rgba(" + N[lo].join(",") + "," + a + ")";
    if (color[0] === "#") {
      var h = color.slice(1);
      if (h.length === 3) h = h[0] + h[0] + h[1] + h[1] + h[2] + h[2];
      return (
        "rgba(" +
        parseInt(h.slice(0, 2), 16) +
        "," +
        parseInt(h.slice(2, 4), 16) +
        "," +
        parseInt(h.slice(4, 6), 16) +
        "," +
        a +
        ")"
      );
    }
    return color;
  }

  // ── DOM helpers ──

  function mkTitle(text) {
    var el = document.createElement("div");
    el.className = "graph-chart-title";
    el.textContent = text;
    return el;
  }

  function mkLegend(items) {
    var wrap = document.createElement("div");
    wrap.className = "graph-legend";
    items.forEach(function (item) {
      var pill = document.createElement("span");
      pill.className = "graph-legend-item";
      var dot = document.createElement("span");
      dot.className = "graph-legend-dot";
      dot.style.backgroundColor = item.color;
      pill.appendChild(dot);
      var labelSpan = document.createElement("span");
      if (item.katex && typeof katex !== "undefined") {
        try {
          katex.render(item.katex, labelSpan, {
            throwOnError: false,
            displayMode: false,
          });
        } catch (e) {
          labelSpan.textContent = item.katex;
        }
      } else {
        labelSpan.textContent = item.label;
      }
      pill.appendChild(labelSpan);
      wrap.appendChild(pill);
    });
    return wrap;
  }

  // ── function-plot ──

  function loadFP(cb) {
    if (typeof functionPlot !== "undefined") return cb();
    var s = document.createElement("script");
    s.src =
      "https://cdn.jsdelivr.net/npm/function-plot@1/dist/function-plot.js";
    s.onload = cb;
    s.onerror = function () {
      console.error("Failed to load function-plot");
    };
    document.head.appendChild(s);
  }

  function mkSvgIcon(pathD, extra) {
    var ns = "http://www.w3.org/2000/svg";
    var svg = document.createElementNS(ns, "svg");
    svg.setAttribute("width", "14");
    svg.setAttribute("height", "14");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "2");
    (pathD || []).forEach(function (d) {
      var tag = d[0];
      var el = document.createElementNS(ns, tag);
      var attrs = d[1];
      for (var k in attrs) el.setAttribute(k, attrs[k]);
      svg.appendChild(el);
    });
    return svg;
  }

  // Block D3's wheel-zoom so page scroll works normally over graphs.
  // D3 adds a wheel listener that calls preventDefault() — we intercept
  // it in the capture phase first and stop it from reaching D3.
  function blockWheelZoom(container) {
    var svg = container.querySelector("svg");
    if (svg) {
      svg.addEventListener(
        "wheel",
        function (e) {
          e.stopImmediatePropagation();
        },
        true,
      );
    }
  }

  // Block D3 touch drag on mobile — prevents hijacking page scroll.
  function blockDrag(container) {
    var svg = container.querySelector("svg");
    if (svg) {
      svg.addEventListener(
        "touchstart",
        function (e) {
          e.stopImmediatePropagation();
        },
        true,
      );
      svg.addEventListener(
        "touchmove",
        function (e) {
          e.stopImmediatePropagation();
        },
        true,
      );
    }
  }

  function renderGraph(el) {
    var raw = el.textContent;
    el.setAttribute("data-config", raw);
    var cfg = parseDSL(raw);
    var p = pal();

    var xDom = cfg.x ? parseRange(cfg.x) : [-10, 10];
    var yDom = cfg.y ? parseRange(cfg.y) : [-10, 10];
    var origXDom = xDom.slice();
    var origYDom = yDom.slice();

    var fns = (cfg.fn || []).map(function (entry, i) {
      var parts = entry.split("|");
      var expr = parts[0].trim();
      var colorStr = parts[1] ? parts[1].trim() : null;
      var labelStr = parts[2] ? parts[2].trim() : null;
      return {
        expr: expr,
        color: pickColor(colorStr || null, i, p),
        label: expr,
        katex: labelStr || null,
      };
    });

    el.textContent = "";

    if (cfg.title) el.appendChild(mkTitle(cfg.title));

    // Graph wrapper — no overflow restriction so D3 zoom/pan works
    var graphWrap = document.createElement("div");
    graphWrap.className = "graph-plot-wrap";
    el.appendChild(graphWrap);

    var target = document.createElement("div");
    target.className = "graph-plot-target";
    graphWrap.appendChild(target);

    var plotData = fns.map(function (f) {
      return { fn: f.expr, graphType: "polyline", color: f.color };
    });

    // Responsive dimensions — recalculated on each render/replot
    function getPlotDims() {
      var m = isMobile();
      var pad = m ? 4 : 48;
      var w = el.clientWidth - pad;
      return { width: Math.max(w, 200), height: m ? 220 : 360 };
    }

    var plotInstance;
    try {
      var dims = getPlotDims();
      plotInstance = functionPlot({
        target: target,
        width: dims.width,
        height: dims.height,
        xAxis: { domain: xDom, label: cfg.xlabel || "" },
        yAxis: { domain: yDom, label: cfg.ylabel || "" },
        grid: true,
        data: plotData,
      });
      blockWheelZoom(target);
      // Disable D3 drag on mobile — conflicts with page scroll
      if (isMobile()) blockDrag(target);
    } catch (e) {
      console.error("function-plot error:", e);
      target.style.padding = "2rem";
      target.style.textAlign = "center";
      target.style.color = p.textMuted;
      target.style.fontSize = "0.8125rem";
      target.textContent = "Graph error: " + e.message;
      return;
    }

    // ── Zoom/pan control buttons ──
    var controls = document.createElement("div");
    controls.className = "graph-controls";

    // + button
    var btnZoomIn = document.createElement("button");
    btnZoomIn.className = "graph-ctrl-btn";
    btnZoomIn.title = "Zoom in";
    btnZoomIn.setAttribute("aria-label", "Zoom in");
    btnZoomIn.appendChild(
      mkSvgIcon([
        ["line", { x1: "12", y1: "5", x2: "12", y2: "19" }],
        ["line", { x1: "5", y1: "12", x2: "19", y2: "12" }],
      ]),
    );

    // - button
    var btnZoomOut = document.createElement("button");
    btnZoomOut.className = "graph-ctrl-btn";
    btnZoomOut.title = "Zoom out";
    btnZoomOut.setAttribute("aria-label", "Zoom out");
    btnZoomOut.appendChild(
      mkSvgIcon([["line", { x1: "5", y1: "12", x2: "19", y2: "12" }]]),
    );

    // reset button
    var btnReset = document.createElement("button");
    btnReset.className = "graph-ctrl-btn";
    btnReset.title = "Reset view";
    btnReset.setAttribute("aria-label", "Reset view");
    btnReset.appendChild(
      mkSvgIcon([
        ["path", { d: "M3 12a9 9 0 1 1 3 6.7" }],
        ["polyline", { points: "3 20 3 13 10 13" }],
      ]),
    );

    controls.appendChild(btnZoomIn);
    controls.appendChild(btnZoomOut);
    controls.appendChild(btnReset);
    graphWrap.appendChild(controls);

    // Current domain state for button zoom
    var curXDom = xDom.slice();
    var curYDom = yDom.slice();

    // Track domain changes from D3 mouse zoom/pan
    plotInstance.on("all:zoom", function (d) {
      curXDom = plotInstance.meta.xScale.domain().slice();
      curYDom = plotInstance.meta.yScale.domain().slice();
    });

    function replot(newX, newY) {
      curXDom = newX;
      curYDom = newY;
      target.textContent = "";
      var dims = getPlotDims();
      plotInstance = functionPlot({
        target: target,
        width: dims.width,
        height: dims.height,
        xAxis: { domain: curXDom, label: cfg.xlabel || "" },
        yAxis: { domain: curYDom, label: cfg.ylabel || "" },
        grid: true,
        data: plotData,
      });
      blockWheelZoom(target);
      if (isMobile()) blockDrag(target);
      plotInstance.on("all:zoom", function () {
        curXDom = plotInstance.meta.xScale.domain().slice();
        curYDom = plotInstance.meta.yScale.domain().slice();
      });
    }

    function zoomBy(factor) {
      var cx = (curXDom[0] + curXDom[1]) / 2;
      var cy = (curYDom[0] + curYDom[1]) / 2;
      var xH = ((curXDom[1] - curXDom[0]) / 2) * factor;
      var yH = ((curYDom[1] - curYDom[0]) / 2) * factor;
      replot([cx - xH, cx + xH], [cy - yH, cy + yH]);
    }

    btnZoomIn.addEventListener("click", function () {
      zoomBy(0.6);
    });
    btnZoomOut.addEventListener("click", function () {
      zoomBy(1.6);
    });
    btnReset.addEventListener("click", function () {
      replot(origXDom.slice(), origYDom.slice());
    });

    // Custom legend pills (with optional KaTeX labels)
    if (fns.length > 0) {
      el.appendChild(
        mkLegend(
          fns.map(function (f) {
            return { label: f.label, color: f.color, katex: f.katex };
          }),
        ),
      );
    }
  }

  // ── Chart.js ──

  function loadCJ(cb) {
    if (typeof Chart !== "undefined") return cb();
    var s = document.createElement("script");
    s.src = "https://cdn.jsdelivr.net/npm/chart.js@4/dist/chart.umd.min.js";
    s.onload = cb;
    s.onerror = function () {
      console.error("Failed to load Chart.js");
    };
    document.head.appendChild(s);
  }

  function renderChart(el) {
    var raw = el.textContent;
    el.setAttribute("data-config", raw);
    var cfg = parseDSL(raw);
    var p = pal();
    var dark = isDark();

    var type = (cfg.type || "line").toLowerCase();
    var labels = (cfg.labels || "").split(",").map(function (s) {
      return s.trim();
    });
    var isPie = type === "pie" || type === "doughnut";

    var datasets = (cfg.dataset || []).map(function (entry, di) {
      var parts = entry.split("|");
      var label = (parts[0] || "").trim();
      var vals = (parts[1] || "")
        .trim()
        .split(",")
        .map(function (v) {
          return parseFloat(v.trim());
        });
      var userColor = parts[2] ? parts[2].trim() : null;
      var color = pickColor(userColor, di, p);

      if (isPie) {
        return {
          label: label,
          data: vals,
          borderColor: dark ? "rgba(28,24,26,0.6)" : "rgba(255,255,255,0.8)",
          borderWidth: 2,
          backgroundColor: vals.map(function (_, i) {
            return toRgba(pickColor(null, i, p), 0.65);
          }),
          hoverBackgroundColor: vals.map(function (_, i) {
            return toRgba(pickColor(null, i, p), 0.85);
          }),
        };
      }

      var ds = {
        label: label,
        data: vals,
        borderColor: color,
        borderWidth: 2.5,
        tension: 0.35,
      };

      if (type === "bar") {
        ds.backgroundColor = toRgba(color, 0.5);
        ds.hoverBackgroundColor = toRgba(color, 0.7);
        ds.borderWidth = 0;
        ds.borderRadius = 6;
        ds.borderSkipped = false;
      } else if (type === "line") {
        ds.backgroundColor = toRgba(color, 0.08);
        ds.fill = true;
        ds.pointRadius = 3;
        ds.pointBackgroundColor = color;
        ds.pointBorderColor = dark ? "#1c181a" : "#fcfafb";
        ds.pointBorderWidth = 1.5;
        ds.pointHoverRadius = 5;
        ds.pointHoverBorderWidth = 2;
      } else if (type === "radar") {
        ds.backgroundColor = toRgba(color, 0.1);
        ds.fill = true;
        ds.pointRadius = 3;
        ds.pointBackgroundColor = color;
        ds.pointBorderColor = dark ? "#1c181a" : "#fcfafb";
        ds.pointBorderWidth = 1.5;
      }

      return ds;
    });

    el.textContent = "";

    if (cfg.title) el.appendChild(mkTitle(cfg.title));

    var wrap = document.createElement("div");
    wrap.className = "chart-canvas-wrap";
    el.appendChild(wrap);

    var canvas = document.createElement("canvas");
    wrap.appendChild(canvas);

    var mobile = isMobile();
    var scalesConf = {};
    if (!isPie) {
      var tickSize = mobile ? 10 : 12;
      var tickPad = mobile ? 3 : 6;
      var axisBase = {
        ticks: {
          color: p.textMuted,
          font: {
            family: "'JetBrains Mono', monospace",
            size: tickSize,
            weight: "400",
          },
          padding: tickPad,
        },
        grid: {
          color: p.grid,
          lineWidth: 1,
          drawTicks: false,
        },
        border: {
          display: false,
        },
      };
      if (type === "radar") {
        scalesConf.r = {
          ticks: {
            color: p.textMuted,
            backdropColor: "transparent",
            font: { family: "'JetBrains Mono', monospace", size: tickSize },
          },
          grid: { color: p.grid, lineWidth: 1 },
          angleLines: { color: p.grid },
          pointLabels: {
            color: p.text,
            font: { size: tickSize, weight: "500" },
          },
        };
      } else {
        scalesConf.x = JSON.parse(JSON.stringify(axisBase));
        scalesConf.y = JSON.parse(JSON.stringify(axisBase));
        scalesConf.y.ticks.padding = mobile ? 4 : 8;
      }
    }

    new Chart(canvas, {
      type: type,
      data: { labels: labels, datasets: datasets },
      options: {
        responsive: true,
        maintainAspectRatio: true,
        aspectRatio: mobile ? 1.2 : 2,
        animation: { duration: 700, easing: "easeOutQuart" },
        interaction: { intersect: false, mode: isPie ? "nearest" : "index" },
        layout: {
          padding: mobile
            ? { top: 2, bottom: 2, left: 0, right: 0 }
            : { top: 4, bottom: 4, left: 2, right: 2 },
        },
        plugins: {
          legend: {
            display: true,
            position: isPie ? "bottom" : "top",
            labels: {
              color: p.text,
              font: { size: mobile ? 10 : 12, weight: "500" },
              padding: mobile ? 8 : 14,
              usePointStyle: true,
              pointStyleWidth: mobile ? 6 : 8,
              boxHeight: mobile ? 6 : 7,
            },
          },
          tooltip: {
            backgroundColor: p.tooltipBg,
            titleColor: p.text,
            bodyColor: p.text,
            borderColor: p.border,
            borderWidth: 1,
            cornerRadius: mobile ? 8 : 10,
            padding: mobile
              ? { top: 6, bottom: 6, left: 8, right: 8 }
              : { top: 10, bottom: 10, left: 14, right: 14 },
            bodyFont: {
              size: mobile ? 10 : 12,
              family: "'JetBrains Mono', monospace",
            },
            titleFont: { size: mobile ? 10 : 12, weight: "600" },
            boxPadding: mobile ? 3 : 4,
            caretSize: mobile ? 4 : 5,
            displayColors: true,
            usePointStyle: true,
          },
        },
        scales: scalesConf,
      },
    });
  }

  // ── Plotly.js ──

  function loadPlotly(cb) {
    if (typeof Plotly !== "undefined") return cb();
    var s = document.createElement("script");
    s.src = "https://cdn.jsdelivr.net/npm/plotly.js-dist-min@2/plotly.min.js";
    s.onload = cb;
    s.onerror = function () {
      console.error("Failed to load Plotly.js");
    };
    document.head.appendChild(s);
  }

  // DSL parser for plot3d — supports array keys (vec, dataset)
  function parsePlot3dConfig(text) {
    var c = {};
    text.split("\n").forEach(function (line) {
      line = line.trim();
      if (!line) return;
      var idx = line.indexOf(":");
      if (idx === -1) return;
      var k = line.substring(0, idx).trim().toLowerCase();
      var v = line.substring(idx + 1).trim();
      if (k === "vec" || k === "dataset" || k === "point") {
        (c[k] = c[k] || []).push(v);
      } else {
        c[k] = v;
      }
    });
    return c;
  }

  // Simple math evaluator for surface functions.
  // Expressions come from the blog author's own markdown content (not user input).
  function evalMathExpr(expr, vars) {
    var s = expr
      .replace(/\^/g, "**")
      .replace(/\bsin\b/g, "Math.sin")
      .replace(/\bcos\b/g, "Math.cos")
      .replace(/\btan\b/g, "Math.tan")
      .replace(/\bsqrt\b/g, "Math.sqrt")
      .replace(/\bexp\b/g, "Math.exp")
      .replace(/\blog\b/g, "Math.log")
      .replace(/\babs\b/g, "Math.abs")
      .replace(/\bpow\b/g, "Math.pow")
      .replace(/\bPI\b/gi, "Math.PI")
      .replace(/\bpi\b/g, "Math.PI")
      .replace(/\be\b/g, "Math.E");
    var keys = Object.keys(vars);
    var vals = keys.map(function (k) {
      return vars[k];
    });
    try {
      var fn = new Function(keys.join(","), "return (" + s + ");"); // eslint-disable-line no-new-func
      return fn.apply(null, vals);
    } catch (e) {
      return NaN;
    }
  }

  function plotlyBaseLayout(cfg, p) {
    var dark = isDark();
    var mobile = isMobile();
    return {
      title: cfg.title
        ? {
            text: cfg.title,
            font: {
              family: "'Nunito', 'Gowun Dodum', sans-serif",
              size: mobile ? 12 : 15,
              color: p.text,
              weight: 600,
            },
          }
        : undefined,
      paper_bgcolor: "rgba(0,0,0,0)",
      plot_bgcolor: "rgba(0,0,0,0)",
      font: {
        family: "'JetBrains Mono', monospace",
        size: mobile ? 10 : 12,
        color: p.textMuted,
      },
      margin: mobile
        ? { l: 32, r: 12, t: cfg.title ? 36 : 12, b: 32 }
        : { l: 50, r: 30, t: cfg.title ? 50 : 20, b: 50 },
      showlegend: false,
      hovermode: false,
    };
  }

  function plotlyAxis2d(p) {
    var dark = isDark();
    var mobile = isMobile();
    return {
      gridcolor: dark ? p.grid : "rgba(0,0,0,0.15)",
      gridwidth: mobile ? 1 : 1.5,
      zerolinecolor: dark ? p.border : "rgba(0,0,0,0.4)",
      zerolinewidth: mobile ? 1.5 : 2,
      showline: false,
      tickfont: { size: mobile ? 9 : 12, color: p.textMuted },
    };
  }

  function plotlyAxis3d(p) {
    var dark = isDark();
    var mobile = isMobile();
    return {
      backgroundcolor: "rgba(0,0,0,0)",
      gridcolor: dark ? p.grid : "rgba(0,0,0,0.2)",
      gridwidth: mobile ? 1 : 2,
      zerolinecolor: dark ? p.border : "rgba(0,0,0,0.4)",
      zerolinewidth: mobile ? 1.5 : 2,
      showbackground: false,
      showline: false,
      tickfont: {
        size: mobile ? 9 : 12,
        color: dark ? p.textMuted : "rgba(0,0,0,0.7)",
      },
    };
  }

  // Block Plotly's wheel-zoom on 3D canvases so page scroll works normally.
  function blockPlotlyWheelZoom(container) {
    var canvas = container.querySelector("canvas");
    if (canvas) {
      canvas.addEventListener(
        "wheel",
        function (e) {
          e.stopImmediatePropagation();
        },
        true,
      );
    }
  }

  // Shared Plotly config — no default UI, scroll-zoom blocked
  var PLOTLY_CFG_2D = {
    responsive: true,
    displaylogo: false,
    displayModeBar: false,
    scrollZoom: false,
    doubleClick: false,
  };

  var PLOTLY_CFG_3D = {
    responsive: true,
    displaylogo: false,
    displayModeBar: false,
    scrollZoom: false,
  };

  // Custom zoom / reset controls (same look as graph block)
  function addPlotlyControls(graphWrap, opts) {
    var controls = document.createElement("div");
    controls.className = "graph-controls";

    var btnZoomIn = document.createElement("button");
    btnZoomIn.className = "graph-ctrl-btn";
    btnZoomIn.title = "Zoom in";
    btnZoomIn.setAttribute("aria-label", "Zoom in");
    btnZoomIn.appendChild(
      mkSvgIcon([
        ["line", { x1: "12", y1: "5", x2: "12", y2: "19" }],
        ["line", { x1: "5", y1: "12", x2: "19", y2: "12" }],
      ]),
    );

    var btnZoomOut = document.createElement("button");
    btnZoomOut.className = "graph-ctrl-btn";
    btnZoomOut.title = "Zoom out";
    btnZoomOut.setAttribute("aria-label", "Zoom out");
    btnZoomOut.appendChild(
      mkSvgIcon([["line", { x1: "5", y1: "12", x2: "19", y2: "12" }]]),
    );

    var btnReset = document.createElement("button");
    btnReset.className = "graph-ctrl-btn";
    btnReset.title = "Reset view";
    btnReset.setAttribute("aria-label", "Reset view");
    btnReset.appendChild(
      mkSvgIcon([
        ["path", { d: "M3 12a9 9 0 1 1 3 6.7" }],
        ["polyline", { points: "3 20 3 13 10 13" }],
      ]),
    );

    controls.appendChild(btnZoomIn);
    controls.appendChild(btnZoomOut);
    controls.appendChild(btnReset);
    graphWrap.appendChild(controls);

    btnZoomIn.addEventListener("click", function () {
      opts.zoomIn();
    });
    btnZoomOut.addEventListener("click", function () {
      opts.zoomOut();
    });
    btnReset.addEventListener("click", function () {
      opts.reset();
    });
  }

  function renderPlot3dSurface(el, cfg, p) {
    var xRange = cfg.x ? parseRange(cfg.x) : [-5, 5];
    var yRange = cfg.y ? parseRange(cfg.y) : [-5, 5];
    var expr = cfg.fn || "sin(sqrt(x^2 + y^2))";

    var N = isMobile() ? 40 : 60;
    var xVals = [],
      yVals = [],
      zVals = [];
    var dx = (xRange[1] - xRange[0]) / N;
    var dy = (yRange[1] - yRange[0]) / N;

    for (var i = 0; i <= N; i++) {
      xVals.push(xRange[0] + i * dx);
    }
    for (var j = 0; j <= N; j++) {
      yVals.push(yRange[0] + j * dy);
    }
    for (var j = 0; j <= N; j++) {
      var row = [];
      for (var i = 0; i <= N; i++) {
        row.push(evalMathExpr(expr, { x: xVals[i], y: yVals[j] }));
      }
      zVals.push(row);
    }

    var dark = isDark();
    var trace = {
      type: "surface",
      x: xVals,
      y: yVals,
      z: zVals,
      colorscale: dark
        ? [
            [0, "#2a1f2d"],
            [0.25, "#6b4a6e"],
            [0.5, "#b096b8"],
            [0.75, "#d4a89e"],
            [1, "#e8d0c8"],
          ]
        : [
            [0, "#8db8a0"],
            [0.25, "#b096b8"],
            [0.5, "#c9899e"],
            [0.75, "#d4a89e"],
            [1, "#d4c098"],
          ],
      showscale: true,
      colorbar: {
        tickfont: {
          size: isMobile() ? 9 : 12,
          color: isDark() ? p.textMuted : "rgba(0,0,0,0.7)",
        },
        thickness: isMobile() ? 10 : 15,
        len: isMobile() ? 0.5 : 0.6,
      },
    };

    var layout = plotlyBaseLayout(cfg, p);
    layout.scene = {
      xaxis: plotlyAxis3d(p),
      yaxis: plotlyAxis3d(p),
      zaxis: plotlyAxis3d(p),
    };

    var graphWrap = document.createElement("div");
    graphWrap.className = "graph-plot-wrap";
    el.appendChild(graphWrap);

    var wrap = document.createElement("div");
    graphWrap.appendChild(wrap);
    Plotly.newPlot(wrap, [trace], layout, PLOTLY_CFG_3D);
    blockPlotlyWheelZoom(wrap);

    addPlotlyControls(graphWrap, {
      zoomIn: function () {
        var cam = wrap._fullLayout.scene._scene.getCamera();
        cam.eye.x *= 0.75;
        cam.eye.y *= 0.75;
        cam.eye.z *= 0.75;
        Plotly.relayout(wrap, { "scene.camera": cam });
      },
      zoomOut: function () {
        var cam = wrap._fullLayout.scene._scene.getCamera();
        cam.eye.x *= 1.35;
        cam.eye.y *= 1.35;
        cam.eye.z *= 1.35;
        Plotly.relayout(wrap, { "scene.camera": cam });
      },
      reset: function () {
        Plotly.relayout(wrap, {
          "scene.camera": { eye: { x: 1.25, y: 1.25, z: 1.25 } },
        });
      },
    });
  }

  function renderPlot3dVector2d(el, cfg, p) {
    var xRange = cfg.x ? parseRange(cfg.x) : [-1, 4];
    var yRange = cfg.y ? parseRange(cfg.y) : [-1, 4];
    var vecs = cfg.vec || [];

    var traces = [];
    var annotations = [];
    var legendItems = [];

    vecs.forEach(function (entry, idx) {
      var parts = entry.split("|");
      var coords = parts[0]
        .trim()
        .split(",")
        .map(function (v) {
          return parseFloat(v.trim());
        });
      var colorStr = parts[1] ? parts[1].trim() : "";
      var color = pickColor(colorStr || null, idx, p);
      var label = parts[2] ? parts[2].trim() : "";

      var vx = coords[0] || 0;
      var vy = coords[1] || 0;

      legendItems.push({
        label: label || "(" + vx + "," + vy + ")",
        color: color,
        katex: label || null,
      });

      traces.push({
        type: "scatter",
        x: [0, vx],
        y: [0, vy],
        mode: "lines",
        line: { color: color, width: 2.5 },
      });

      // Arrowhead annotation
      annotations.push({
        x: vx,
        y: vy,
        ax: vx * 0.7,
        ay: vy * 0.7,
        xref: "x",
        yref: "y",
        axref: "x",
        ayref: "y",
        showarrow: true,
        arrowhead: 3,
        arrowsize: 1.5,
        arrowwidth: 2.5,
        arrowcolor: color,
      });
    });

    var layout = plotlyBaseLayout(cfg, p);
    layout.xaxis = plotlyAxis2d(p);
    layout.yaxis = plotlyAxis2d(p);
    layout.xaxis.range = xRange;
    layout.yaxis.range = yRange;
    layout.xaxis.scaleanchor = "y";
    layout.xaxis.scaleratio = 1;
    layout.dragmode = "pan";
    layout.annotations = annotations;

    var graphWrap = document.createElement("div");
    graphWrap.className = "graph-plot-wrap";
    el.appendChild(graphWrap);

    var wrap = document.createElement("div");
    graphWrap.appendChild(wrap);
    Plotly.newPlot(wrap, traces, layout, PLOTLY_CFG_2D);

    var curX = xRange.slice();
    var curY = yRange.slice();

    function relayout2d(newX, newY) {
      curX = newX;
      curY = newY;
      Plotly.relayout(wrap, {
        "xaxis.range": curX,
        "yaxis.range": curY,
      });
    }

    addPlotlyControls(graphWrap, {
      zoomIn: function () {
        var cx = (curX[0] + curX[1]) / 2;
        var cy = (curY[0] + curY[1]) / 2;
        var xH = ((curX[1] - curX[0]) / 2) * 0.6;
        var yH = ((curY[1] - curY[0]) / 2) * 0.6;
        relayout2d([cx - xH, cx + xH], [cy - yH, cy + yH]);
      },
      zoomOut: function () {
        var cx = (curX[0] + curX[1]) / 2;
        var cy = (curY[0] + curY[1]) / 2;
        var xH = ((curX[1] - curX[0]) / 2) * 1.6;
        var yH = ((curY[1] - curY[0]) / 2) * 1.6;
        relayout2d([cx - xH, cx + xH], [cy - yH, cy + yH]);
      },
      reset: function () {
        relayout2d(xRange.slice(), yRange.slice());
      },
    });

    if (legendItems.length) el.appendChild(mkLegend(legendItems));
  }

  function renderPlot3dPoint2d(el, cfg, p) {
    var xRange = cfg.x ? parseRange(cfg.x) : [-1, 4];
    var yRange = cfg.y ? parseRange(cfg.y) : [-1, 4];
    var points = cfg.point || [];

    var traces = [];
    var legendItems = [];

    points.forEach(function (entry, idx) {
      var parts = entry.split("|");
      var coords = parts[0]
        .trim()
        .split(",")
        .map(function (v) {
          return parseFloat(v.trim());
        });
      var colorStr = parts[1] ? parts[1].trim() : "";
      var color = pickColor(colorStr || null, idx, p);
      var label = parts[2] ? parts[2].trim() : "";

      var px = coords[0] || 0;
      var py = coords[1] || 0;

      legendItems.push({
        label: label || "(" + px + "," + py + ")",
        color: color,
        katex: label || null,
      });

      traces.push({
        type: "scatter",
        x: [px],
        y: [py],
        mode: "markers",
        marker: {
          size: 9,
          color: color,
          opacity: 1,
          line: {
            width: 1,
            color: isDark() ? "rgba(255,255,255,0.2)" : "rgba(0,0,0,0.2)",
          },
        },
      });
    });

    var layout = plotlyBaseLayout(cfg, p);
    layout.xaxis = plotlyAxis2d(p);
    layout.yaxis = plotlyAxis2d(p);
    layout.xaxis.range = xRange;
    layout.yaxis.range = yRange;
    layout.xaxis.scaleanchor = "y";
    layout.xaxis.scaleratio = 1;
    layout.dragmode = "pan";

    var graphWrap = document.createElement("div");
    graphWrap.className = "graph-plot-wrap";
    el.appendChild(graphWrap);

    var wrap = document.createElement("div");
    graphWrap.appendChild(wrap);
    Plotly.newPlot(wrap, traces, layout, PLOTLY_CFG_2D);

    var curX = xRange.slice();
    var curY = yRange.slice();

    function relayout2d(newX, newY) {
      curX = newX;
      curY = newY;
      Plotly.relayout(wrap, {
        "xaxis.range": curX,
        "yaxis.range": curY,
      });
    }

    addPlotlyControls(graphWrap, {
      zoomIn: function () {
        var cx = (curX[0] + curX[1]) / 2;
        var cy = (curY[0] + curY[1]) / 2;
        var xH = ((curX[1] - curX[0]) / 2) * 0.6;
        var yH = ((curY[1] - curY[0]) / 2) * 0.6;
        relayout2d([cx - xH, cx + xH], [cy - yH, cy + yH]);
      },
      zoomOut: function () {
        var cx = (curX[0] + curX[1]) / 2;
        var cy = (curY[0] + curY[1]) / 2;
        var xH = ((curX[1] - curX[0]) / 2) * 1.6;
        var yH = ((curY[1] - curY[0]) / 2) * 1.6;
        relayout2d([cx - xH, cx + xH], [cy - yH, cy + yH]);
      },
      reset: function () {
        relayout2d(xRange.slice(), yRange.slice());
      },
    });

    if (legendItems.length) el.appendChild(mkLegend(legendItems));
  }

  function renderPlot3dTransform2d(el, cfg, p) {
    // Parse 2x2 matrix: "a, b, c, d" → [[a, b], [c, d]]
    var m = (cfg.matrix || "1,0,0,1").split(",").map(function (v) {
      return parseFloat(v.trim());
    });
    var a = m[0] || 0,
      b = m[1] || 0,
      c = m[2] || 0,
      d = m[3] || 0;

    // Parse grid range and step
    var gr = cfg.grid ? parseRange(cfg.grid) : [-2, 2];
    var step = cfg.step ? parseFloat(cfg.step) : 1;

    // Generate grid points: before (z=0) → after (z=1)
    var pts = [];
    for (var gx = gr[0]; gx <= gr[1]; gx += step) {
      for (var gy = gr[0]; gy <= gr[1]; gy += step) {
        var tx = a * gx + b * gy;
        var ty = c * gx + d * gy;
        pts.push({ x0: gx, y0: gy, x1: tx, y1: ty });
      }
    }

    var dark = isDark();
    var beforeColor = p.series[2];
    var afterColor = p.series[0];
    var lineColor = toRgba(p.series[1], dark ? 0.6 : 0.7);

    var traces = [];

    // Connection lines (before z=0 → after z=1)
    pts.forEach(function (pt) {
      traces.push({
        type: "scatter3d",
        x: [pt.x0, pt.x1],
        y: [pt.y0, pt.y1],
        z: [0, 1],
        mode: "lines",
        line: { color: lineColor, width: 3 },
        showlegend: false,
      });
    });

    // Before points (z=0)
    traces.push({
      type: "scatter3d",
      x: pts.map(function (pt) {
        return pt.x0;
      }),
      y: pts.map(function (pt) {
        return pt.y0;
      }),
      z: pts.map(function () {
        return 0;
      }),
      mode: "markers",
      marker: {
        size: 6,
        color: beforeColor,
        opacity: 1,
        line: {
          width: 0.5,
          color: dark ? "rgba(255,255,255,0.2)" : "rgba(0,0,0,0.2)",
        },
      },
      showlegend: false,
    });

    // After points (z=1)
    traces.push({
      type: "scatter3d",
      x: pts.map(function (pt) {
        return pt.x1;
      }),
      y: pts.map(function (pt) {
        return pt.y1;
      }),
      z: pts.map(function () {
        return 1;
      }),
      mode: "markers",
      marker: {
        size: 6,
        color: afterColor,
        opacity: 1,
        line: {
          width: 0.5,
          color: dark ? "rgba(255,255,255,0.2)" : "rgba(0,0,0,0.2)",
        },
      },
      showlegend: false,
    });

    var layout = plotlyBaseLayout(cfg, p);
    layout.scene = {
      xaxis: plotlyAxis3d(p),
      yaxis: plotlyAxis3d(p),
      zaxis: Object.assign({}, plotlyAxis3d(p), {
        tickvals: [0, 1],
        ticktext: [
          _i18n.graph_before || "Before",
          _i18n.graph_after || "After",
        ],
      }),
      camera: { eye: { x: 1.6, y: -1.6, z: 1.0 } },
    };

    var graphWrap = document.createElement("div");
    graphWrap.className = "graph-plot-wrap";
    el.appendChild(graphWrap);

    var wrap = document.createElement("div");
    graphWrap.appendChild(wrap);
    Plotly.newPlot(wrap, traces, layout, PLOTLY_CFG_3D);
    blockPlotlyWheelZoom(wrap);

    addPlotlyControls(graphWrap, {
      zoomIn: function () {
        var cam = wrap._fullLayout.scene._scene.getCamera();
        cam.eye.x *= 0.75;
        cam.eye.y *= 0.75;
        cam.eye.z *= 0.75;
        Plotly.relayout(wrap, { "scene.camera": cam });
      },
      zoomOut: function () {
        var cam = wrap._fullLayout.scene._scene.getCamera();
        cam.eye.x *= 1.35;
        cam.eye.y *= 1.35;
        cam.eye.z *= 1.35;
        Plotly.relayout(wrap, { "scene.camera": cam });
      },
      reset: function () {
        Plotly.relayout(wrap, {
          "scene.camera": { eye: { x: 1.6, y: -1.6, z: 1.0 } },
        });
      },
    });

    // Legend
    el.appendChild(
      mkLegend([
        {
          label: _i18n.graph_before || "Before",
          color: beforeColor,
          katex: null,
        },
        { label: _i18n.graph_after || "After", color: afterColor, katex: null },
      ]),
    );
  }

  function renderPlot3dVector3d(el, cfg, p) {
    var vecs = cfg.vec || [];
    var traces = [];
    var legendItems = [];

    vecs.forEach(function (entry, idx) {
      var parts = entry.split("|");
      var coords = parts[0]
        .trim()
        .split(",")
        .map(function (v) {
          return parseFloat(v.trim());
        });
      var colorStr = parts[1] ? parts[1].trim() : "";
      var color = pickColor(colorStr || null, idx, p);
      var label = parts[2] ? parts[2].trim() : "";

      var vx = coords[0] || 0;
      var vy = coords[1] || 0;
      var vz = coords[2] || 0;

      legendItems.push({
        label: label || "(" + vx + "," + vy + "," + vz + ")",
        color: color,
        katex: label || null,
      });

      traces.push({
        type: "scatter3d",
        x: [0, vx],
        y: [0, vy],
        z: [0, vz],
        mode: "lines",
        line: { color: color, width: 5 },
      });

      // Cone arrowhead at tip
      traces.push({
        type: "cone",
        x: [vx],
        y: [vy],
        z: [vz],
        u: [vx * 0.15],
        v: [vy * 0.15],
        w: [vz * 0.15],
        colorscale: [
          [0, color],
          [1, color],
        ],
        showscale: false,
        sizemode: "absolute",
        sizeref: 0.3,
        anchor: "tip",
        showlegend: false,
      });
    });

    var layout = plotlyBaseLayout(cfg, p);
    layout.scene = {
      xaxis: plotlyAxis3d(p),
      yaxis: plotlyAxis3d(p),
      zaxis: plotlyAxis3d(p),
      aspectmode: "data",
    };

    var graphWrap = document.createElement("div");
    graphWrap.className = "graph-plot-wrap";
    el.appendChild(graphWrap);

    var wrap = document.createElement("div");
    graphWrap.appendChild(wrap);
    Plotly.newPlot(wrap, traces, layout, PLOTLY_CFG_3D);
    blockPlotlyWheelZoom(wrap);

    addPlotlyControls(graphWrap, {
      zoomIn: function () {
        var cam = wrap._fullLayout.scene._scene.getCamera();
        cam.eye.x *= 0.75;
        cam.eye.y *= 0.75;
        cam.eye.z *= 0.75;
        Plotly.relayout(wrap, { "scene.camera": cam });
      },
      zoomOut: function () {
        var cam = wrap._fullLayout.scene._scene.getCamera();
        cam.eye.x *= 1.35;
        cam.eye.y *= 1.35;
        cam.eye.z *= 1.35;
        Plotly.relayout(wrap, { "scene.camera": cam });
      },
      reset: function () {
        Plotly.relayout(wrap, {
          "scene.camera": { eye: { x: 1.25, y: 1.25, z: 1.25 } },
        });
      },
    });

    if (legendItems.length) el.appendChild(mkLegend(legendItems));
  }

  function renderPlot3dScatter3d(el, cfg, p) {
    var datasets = cfg.dataset || [];
    var traces = [];
    var legendItems = [];

    datasets.forEach(function (entry, di) {
      var parts = entry.split("|");
      var label = (parts[0] || "").trim();
      var pointsStr = (parts[1] || "").trim();
      var colorStr2 = parts[2] ? parts[2].trim() : "";
      var color = pickColor(colorStr2 || null, di, p);

      legendItems.push({ label: label, color: color, katex: null });

      var xs = [],
        ys = [],
        zs = [];
      pointsStr.split(";").forEach(function (pt) {
        var coords = pt
          .trim()
          .split(",")
          .map(function (v) {
            return parseFloat(v.trim());
          });
        if (coords.length >= 3) {
          xs.push(coords[0]);
          ys.push(coords[1]);
          zs.push(coords[2]);
        }
      });

      traces.push({
        type: "scatter3d",
        mode: "markers",
        x: xs,
        y: ys,
        z: zs,
        marker: {
          size: 7,
          color: color,
          opacity: 1,
          line: {
            width: 0.5,
            color: isDark() ? "rgba(255,255,255,0.2)" : "rgba(0,0,0,0.2)",
          },
        },
      });
    });

    var layout = plotlyBaseLayout(cfg, p);
    layout.scene = {
      xaxis: plotlyAxis3d(p),
      yaxis: plotlyAxis3d(p),
      zaxis: plotlyAxis3d(p),
    };

    var graphWrap = document.createElement("div");
    graphWrap.className = "graph-plot-wrap";
    el.appendChild(graphWrap);

    var wrap = document.createElement("div");
    graphWrap.appendChild(wrap);
    Plotly.newPlot(wrap, traces, layout, PLOTLY_CFG_3D);
    blockPlotlyWheelZoom(wrap);

    addPlotlyControls(graphWrap, {
      zoomIn: function () {
        var cam = wrap._fullLayout.scene._scene.getCamera();
        cam.eye.x *= 0.75;
        cam.eye.y *= 0.75;
        cam.eye.z *= 0.75;
        Plotly.relayout(wrap, { "scene.camera": cam });
      },
      zoomOut: function () {
        var cam = wrap._fullLayout.scene._scene.getCamera();
        cam.eye.x *= 1.35;
        cam.eye.y *= 1.35;
        cam.eye.z *= 1.35;
        Plotly.relayout(wrap, { "scene.camera": cam });
      },
      reset: function () {
        Plotly.relayout(wrap, {
          "scene.camera": { eye: { x: 1.25, y: 1.25, z: 1.25 } },
        });
      },
    });

    if (legendItems.length) el.appendChild(mkLegend(legendItems));
  }

  function renderPlot3d(el) {
    var raw = el.textContent;
    el.setAttribute("data-config", raw);
    var cfg = parsePlot3dConfig(raw);
    var p = pal();
    var type = (cfg.type || "surface").toLowerCase();

    el.textContent = "";

    if (type === "surface") {
      renderPlot3dSurface(el, cfg, p);
    } else if (type === "vector2d") {
      renderPlot3dVector2d(el, cfg, p);
    } else if (type === "point2d") {
      renderPlot3dPoint2d(el, cfg, p);
    } else if (type === "transform2d") {
      renderPlot3dTransform2d(el, cfg, p);
    } else if (type === "vector3d") {
      renderPlot3dVector3d(el, cfg, p);
    } else if (type === "scatter3d") {
      renderPlot3dScatter3d(el, cfg, p);
    }
  }

  // ── Debounce helper ──

  function debounce(fn, ms) {
    var timer;
    return function () {
      clearTimeout(timer);
      timer = setTimeout(fn, ms);
    };
  }

  // ── Theme toggle / resize re-render ──

  function reRender(graphEls, chartEls, plotlyEls) {
    graphEls.forEach(function (el) {
      var cfg = el.getAttribute("data-config");
      if (!cfg) return;
      el.textContent = cfg;
      renderGraph(el);
    });
    chartEls.forEach(function (el) {
      var canvas = el.querySelector("canvas");
      if (canvas) {
        var inst = Chart.getChart(canvas);
        if (inst) inst.destroy();
      }
      var cfg = el.getAttribute("data-config");
      if (!cfg) return;
      el.textContent = cfg;
      renderChart(el);
    });
    plotlyEls.forEach(function (el) {
      var plotDiv = el.querySelector(".js-plotly-plot");
      if (plotDiv) Plotly.purge(plotDiv);
      var cfg = el.getAttribute("data-config");
      if (!cfg) return;
      el.textContent = cfg;
      renderPlot3d(el);
    });
  }

  // ── Init ──

  document.addEventListener("DOMContentLoaded", function () {
    var gEls = document.querySelectorAll(".function-plot-target");
    var cEls = document.querySelectorAll(".chart-js-target");
    var pEls = document.querySelectorAll(".plotly-target");

    // Track async library loads to signal completion
    var pending = 0;
    if (gEls.length) pending++;
    if (cEls.length) pending++;
    if (pEls.length) pending++;

    function onGroupDone() {
      pending--;
      if (pending <= 0) {
        document.dispatchEvent(new Event("graphs:rendered"));
      }
    }

    if (pending === 0) {
      document.dispatchEvent(new Event("graphs:rendered"));
    }

    if (gEls.length) {
      loadFP(function () {
        gEls.forEach(function (el) {
          renderGraph(el);
          el.classList.add("rendered");
        });
        onGroupDone();
      });
    }
    if (cEls.length) {
      loadCJ(function () {
        cEls.forEach(function (el) {
          renderChart(el);
          el.classList.add("rendered");
        });
        onGroupDone();
      });
    }
    if (pEls.length) {
      loadPlotly(function () {
        pEls.forEach(function (el) {
          renderPlot3d(el);
          el.classList.add("rendered");
        });
        onGroupDone();
      });
    }

    // Watch theme changes
    new MutationObserver(function (muts) {
      for (var i = 0; i < muts.length; i++) {
        if (muts[i].attributeName === "data-theme") {
          setTimeout(function () {
            reRender(gEls, cEls, pEls);
          }, 60);
          break;
        }
      }
    }).observe(document.documentElement, { attributes: true });

    // Re-render on resize / orientation change (debounced)
    var hasGraphs = gEls.length || cEls.length || pEls.length;
    if (hasGraphs) {
      var prevW = window.innerWidth;
      window.addEventListener(
        "resize",
        debounce(function () {
          // Only re-render if width actually changed (avoids mobile address-bar triggers)
          if (window.innerWidth !== prevW) {
            prevW = window.innerWidth;
            reRender(gEls, cEls, pEls);
          }
        }, 300),
        { passive: true },
      );
    }
  });
})();
