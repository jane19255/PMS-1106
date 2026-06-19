(function () {
  const table = document.getElementById("billingTable");
  if (!table) return;

  const body = table.tBodies[0];
  const rows = Array.from(body.rows);
  const search = document.getElementById("billingSearch");
  const status = document.getElementById("billingStatus");
  const sort = document.getElementById("billingSort");
  const rowsPerPage = document.getElementById("billingRows");
  const previous = document.getElementById("billingPrevious");
  const next = document.getElementById("billingNext");
  const pageInfo = document.getElementById("billingPageInfo");
  const empty = document.getElementById("billingEmpty");
  let page = 1;

  function filteredRows() {
    const query = search.value.trim().toLowerCase();
    const selectedStatus = status.value;
    const visible = rows.filter((row) =>
      row.dataset.search.toLowerCase().includes(query) &&
      (!selectedStatus || row.dataset.status === selectedStatus)
    );

    return visible.sort((left, right) => {
      if (sort.value === "oldest") return left.dataset.date.localeCompare(right.dataset.date);
      if (sort.value === "amount-high") return Number(right.dataset.amount) - Number(left.dataset.amount);
      if (sort.value === "amount-low") return Number(left.dataset.amount) - Number(right.dataset.amount);
      return right.dataset.date.localeCompare(left.dataset.date);
    });
  }

  function render() {
    const matches = filteredRows();
    const size = Number(rowsPerPage.value);
    const pageCount = Math.max(1, Math.ceil(matches.length / size));
    page = Math.min(page, pageCount);
    const start = (page - 1) * size;
    const shown = matches.slice(start, start + size);

    rows.forEach((row) => { row.hidden = true; });
    shown.forEach((row) => {
      row.hidden = false;
      body.appendChild(row);
    });

    empty.hidden = matches.length !== 0;
    pageInfo.textContent = matches.length
      ? `Showing ${start + 1}–${Math.min(start + size, matches.length)} of ${matches.length}`
      : "Showing 0 invoices";
    previous.disabled = page === 1;
    next.disabled = page === pageCount;
  }

  [search, status, sort, rowsPerPage].forEach((control) => {
    control.addEventListener("input", () => { page = 1; render(); });
  });
  previous.addEventListener("click", () => { page -= 1; render(); });
  next.addEventListener("click", () => { page += 1; render(); });
  render();
})();
