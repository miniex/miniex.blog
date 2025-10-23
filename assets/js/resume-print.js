// Resume Print Optimization
// Automatically adjust layout before printing to prevent awkward page breaks

(function () {
  // Configuration
  const PAGE_HEIGHT_MM = 297; // A4 height in mm
  const PAGE_HEIGHT_PX = (PAGE_HEIGHT_MM / 25.4) * 96; // Convert to pixels (96 DPI)
  const MARGIN_PX = 80; // Top/bottom margins
  const USABLE_PAGE_HEIGHT = PAGE_HEIGHT_PX - MARGIN_PX * 2;

  // Threshold: only push to next page if really necessary
  const BREAK_THRESHOLD = USABLE_PAGE_HEIGHT * 0.5; // 50% of page

  function optimizeForPrint() {
    // Minimal optimization - let browser hndle everything naturally
    console.log("Print layout ready (natural flow)");
  }

  // Run optimization before print
  window.addEventListener("beforeprint", () => {
    console.log("Preparing for print...");
    optimizeForPrint();
  });

  // Clean up after print
  window.addEventListener("afterprint", () => {
    console.log("Print completed");
  });

  // Also optimize on Ctrl+P / Cmd+P
  document.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "p") {
      setTimeout(optimizeForPrint, 100);
    }
  });
})();
