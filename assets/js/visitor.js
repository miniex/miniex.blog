// Visitor tracking
(function () {
  var cid = localStorage.getItem("blog_client_id");
  if (!cid) {
    cid = crypto.randomUUID();
    localStorage.setItem("blog_client_id", cid);
  }
  var now = new Date();
  var localDate =
    now.getFullYear() +
    "-" +
    String(now.getMonth() + 1).padStart(2, "0") +
    "-" +
    String(now.getDate()).padStart(2, "0");

  fetch("/api/visit", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ client_id: cid, date: localDate }),
  }).catch(function () {});

  fetch("/api/visitor-stats?date=" + localDate)
    .then(function (r) {
      return r.json();
    })
    .then(function (data) {
      var todayEl = document.getElementById("visitor-today");
      var totalEl = document.getElementById("visitor-total");
      if (todayEl) todayEl.textContent = data.today;
      if (totalEl) totalEl.textContent = data.total;
    })
    .catch(function () {});
})();
