// Graph & Chart renderer — pastel theme matched
// Lazy-loads function-plot / Chart.js only when targets exist.

(function () {
  "use strict";

  // ── Pastel palettes (light / dark) ──

  var PALETTE_LIGHT = {
    text: "#3d353a",
    textMuted: "rgba(61,53,58,0.4)",
    grid: "rgba(61,53,58,0.05)",
    gridDash: "3,3",
    border: "rgba(61,53,58,0.1)",
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
    textMuted: "rgba(224,216,220,0.38)",
    grid: "rgba(208,160,176,0.06)",
    gridDash: "3,3",
    border: "rgba(208,160,176,0.12)",
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

    var containerW = el.clientWidth - 48;
    var plotW = Math.max(containerW, 320);

    var plotData = fns.map(function (f) {
      return { fn: f.expr, graphType: "polyline", color: f.color };
    });

    var plotInstance;
    try {
      plotInstance = functionPlot({
        target: target,
        width: plotW,
        height: 360,
        xAxis: { domain: xDom, label: cfg.xlabel || "" },
        yAxis: { domain: yDom, label: cfg.ylabel || "" },
        grid: true,
        data: plotData,
      });
      blockWheelZoom(target);
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
    btnZoomOut.appendChild(
      mkSvgIcon([["line", { x1: "5", y1: "12", x2: "19", y2: "12" }]]),
    );

    // reset button
    var btnReset = document.createElement("button");
    btnReset.className = "graph-ctrl-btn";
    btnReset.title = "Reset view";
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
      plotInstance = functionPlot({
        target: target,
        width: plotW,
        height: 360,
        xAxis: { domain: curXDom, label: cfg.xlabel || "" },
        yAxis: { domain: curYDom, label: cfg.ylabel || "" },
        grid: true,
        data: plotData,
      });
      blockWheelZoom(target);
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

    var scalesConf = {};
    if (!isPie) {
      var axisBase = {
        ticks: {
          color: p.textMuted,
          font: {
            family: "'JetBrains Mono', monospace",
            size: 10,
            weight: "400",
          },
          padding: 6,
        },
        grid: {
          color: p.grid,
          lineWidth: 1,
          drawTicks: false,
        },
        border: {
          color: p.border,
          dash: type === "radar" ? undefined : [3, 3],
        },
      };
      if (type === "radar") {
        scalesConf.r = {
          ticks: {
            color: p.textMuted,
            backdropColor: "transparent",
            font: { family: "'JetBrains Mono', monospace", size: 9 },
          },
          grid: { color: p.grid, lineWidth: 1 },
          angleLines: { color: p.grid },
          pointLabels: {
            color: p.text,
            font: { size: 11, weight: "500" },
          },
        };
      } else {
        scalesConf.x = JSON.parse(JSON.stringify(axisBase));
        scalesConf.y = JSON.parse(JSON.stringify(axisBase));
        scalesConf.y.ticks.padding = 8;
      }
    }

    new Chart(canvas, {
      type: type,
      data: { labels: labels, datasets: datasets },
      options: {
        responsive: true,
        maintainAspectRatio: true,
        animation: { duration: 700, easing: "easeOutQuart" },
        interaction: { intersect: false, mode: isPie ? "nearest" : "index" },
        layout: { padding: { top: 4, bottom: 4, left: 2, right: 2 } },
        plugins: {
          legend: {
            display: true,
            position: isPie ? "bottom" : "top",
            labels: {
              color: p.text,
              font: { size: 11, weight: "500" },
              padding: 14,
              usePointStyle: true,
              pointStyleWidth: 8,
              boxHeight: 7,
            },
          },
          tooltip: {
            backgroundColor: p.tooltipBg,
            titleColor: p.text,
            bodyColor: p.text,
            borderColor: p.border,
            borderWidth: 1,
            cornerRadius: 10,
            padding: { top: 10, bottom: 10, left: 14, right: 14 },
            bodyFont: { size: 11, family: "'JetBrains Mono', monospace" },
            titleFont: { size: 11, weight: "600" },
            boxPadding: 4,
            caretSize: 5,
            displayColors: true,
            usePointStyle: true,
          },
        },
        scales: scalesConf,
      },
    });
  }

  // ── Theme toggle re-render ──

  function reRender(graphEls, chartEls) {
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
  }

  // ── Init ──

  document.addEventListener("DOMContentLoaded", function () {
    var gEls = document.querySelectorAll(".function-plot-target");
    var cEls = document.querySelectorAll(".chart-js-target");

    if (gEls.length) {
      loadFP(function () {
        gEls.forEach(renderGraph);
      });
    }
    if (cEls.length) {
      loadCJ(function () {
        cEls.forEach(renderChart);
      });
    }

    // Watch theme changes
    new MutationObserver(function (muts) {
      for (var i = 0; i < muts.length; i++) {
        if (muts[i].attributeName === "data-theme") {
          setTimeout(function () {
            reRender(gEls, cEls);
          }, 60);
          break;
        }
      }
    }).observe(document.documentElement, { attributes: true });
  });
})();
