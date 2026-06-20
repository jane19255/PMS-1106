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
  const invoiceForm = document.querySelector(".invoice-form");
  const patientSearch = document.getElementById("invoicePatientSearch");
  const patientId = document.getElementById("invoicePatientId");
  const patientOptions = document.getElementById("invoicePatientOptions");
  const selectedPatientDetails = document.getElementById("selectedPatientDetails");
  const patientsBySelection = new Map();
  let page = 1;

  function selectPatient(patient) {
    if (!patient) {
      patientId.value = "";
      selectedPatientDetails.textContent = "Select a patient from the database.";
      return;
    }

    const display = `${patient.id} — ${patient.firstName} ${patient.lastName}`;
    patientSearch.value = display;
    patientId.value = patient.id;
    selectedPatientDetails.textContent = [
      `${patient.firstName} ${patient.lastName}`,
      patient.dob ? `DOB: ${patient.dob}` : null,
      patient.phone ? `Phone: ${patient.phone}` : null,
    ].filter(Boolean).join(" | ");
  }

  async function loadPatientOptions() {
    if (!patientSearch) return;

    try {
      const response = await fetch("/api/patients", {
        headers: { Accept: "application/json" },
      });
      if (!response.ok) throw new Error(await response.text());

      const patients = await response.json();
      patients.forEach((patient) => {
        const display = `${patient.id} — ${patient.firstName} ${patient.lastName}`;
        const option = document.createElement("option");
        option.value = display;
        patientOptions.appendChild(option);
        patientsBySelection.set(display.toLowerCase(), patient);
        patientsBySelection.set(String(patient.id).toLowerCase(), patient);
      });

      const requestedId = new URLSearchParams(window.location.search).get("patient_id");
      if (requestedId) {
        selectPatient(patientsBySelection.get(requestedId.toLowerCase()));
        document.querySelector(".create-panel").open = true;
      }
    } catch (error) {
      console.error("Unable to load billing patient selector:", error);
      selectedPatientDetails.textContent = "Patient list could not be loaded.";
    }
  }

  patientSearch?.addEventListener("input", () => {
    patientSearch.setCustomValidity("");
    selectPatient(patientsBySelection.get(patientSearch.value.trim().toLowerCase()));
  });

  invoiceForm?.addEventListener("submit", (event) => {
    if (!patientId.value) {
      event.preventDefault();
      patientSearch.setCustomValidity("Select a patient from the suggestions.");
      patientSearch.reportValidity();
    } else {
      patientSearch.setCustomValidity("");
    }
  });

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
  loadPatientOptions();
  render();
})();
