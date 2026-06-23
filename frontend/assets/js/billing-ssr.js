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
  const prescriptionRows = document.getElementById("prescriptionRows");
  const addPrescriptionRow = document.getElementById("addPrescriptionRow");
  const prescriptionTotal = document.getElementById("prescriptionTotal");
  const patientsBySelection = new Map();
  const CUSTOM_PRESCRIPTION_VALUE = "__custom__";
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
    const duplicateMedicine = findDuplicatePrescription();
    const invalidCustomInput = findInvalidCustomPrescription();
    if (!patientId.value) {
      event.preventDefault();
      patientSearch.setCustomValidity("Select a patient from the suggestions.");
      patientSearch.reportValidity();
    } else if (duplicateMedicine) {
      event.preventDefault();
      const duplicateSelect = Array.from(document.querySelectorAll(".prescription-select"))
        .find((select) => select.value === duplicateMedicine);
      duplicateSelect?.setCustomValidity("This medicine has already been added.");
      duplicateSelect?.reportValidity();
    } else if (invalidCustomInput) {
      event.preventDefault();
      invalidCustomInput.reportValidity();
    } else {
      patientSearch.setCustomValidity("");
    }
  });

  function prescriptionSelects() {
    return Array.from(document.querySelectorAll(".prescription-select"));
  }

  function findDuplicatePrescription() {
    const seen = new Set();
    const customSeen = new Set();
    for (const select of prescriptionSelects()) {
      select.setCustomValidity("");
      const row = select.closest(".prescription-row");
      const customName = row?.querySelector(".custom-medicine-name");
      customName?.setCustomValidity("");

      if (select.value === CUSTOM_PRESCRIPTION_VALUE) {
        const customValue = customName?.value.trim().toLowerCase();
        if (!customValue) continue;
        if (customSeen.has(customValue) || seen.has(customValue)) return CUSTOM_PRESCRIPTION_VALUE;
        customSeen.add(customValue);
        continue;
      }

      if (!select.value) continue;
      const catalogValue = select.value.toLowerCase();
      if (seen.has(catalogValue)) return select.value;
      seen.add(catalogValue);
    }
    return null;
  }

  function findInvalidCustomPrescription() {
    const rows = Array.from(document.querySelectorAll(".prescription-row"));
    for (const row of rows) {
      const select = row.querySelector(".prescription-select");
      const name = row.querySelector(".custom-medicine-name");
      const cost = row.querySelector(".custom-medicine-cost");
      name?.setCustomValidity("");
      cost?.setCustomValidity("");
      if (select?.value !== CUSTOM_PRESCRIPTION_VALUE) continue;

      if (!name?.value.trim()) {
        name?.setCustomValidity("Enter the custom medicine name.");
        return name;
      }
      if (!cost?.value || Number(cost.value) <= 0) {
        cost?.setCustomValidity("Enter a custom medicine cost greater than zero.");
        return cost;
      }
    }
    return null;
  }

  function syncPrescriptionRows() {
    if (!prescriptionRows) return;
    let total = 0;
    const rows = Array.from(prescriptionRows.querySelectorAll(".prescription-row"));

    rows.forEach((row) => {
      const select = row.querySelector(".prescription-select");
      const costInput = row.querySelector(".prescription-cost");
      const customNameLabel = row.querySelector(".custom-prescription-name");
      const customCostLabel = row.querySelector(".custom-prescription-cost");
      const customName = row.querySelector(".custom-medicine-name");
      const customCost = row.querySelector(".custom-medicine-cost");
      const remove = row.querySelector(".remove-prescription-row");
      const isCustom = select?.value === CUSTOM_PRESCRIPTION_VALUE;
      const catalogCost = Number(select?.selectedOptions[0]?.dataset.cost || 0);
      const customAmount = Number(customCost?.value || 0);
      const cost = isCustom ? customAmount : catalogCost;

      if (customNameLabel) customNameLabel.hidden = !isCustom;
      if (customCostLabel) customCostLabel.hidden = !isCustom;
      if (!isCustom) {
        if (customName) customName.value = "";
        if (customCost) customCost.value = "";
      }
      if (costInput) costInput.value = cost ? cost.toFixed(2) : "";
      if (remove) remove.disabled = rows.length === 1;
      total += cost;
    });

    if (prescriptionTotal) prescriptionTotal.textContent = `$${total.toFixed(2)}`;
    findDuplicatePrescription();
  }

  function addMedicineRow() {
    const firstRow = prescriptionRows?.querySelector(".prescription-row");
    if (!firstRow) return;
    const row = firstRow.cloneNode(true);
    row.querySelector(".prescription-select").value = "";
    row.querySelector(".prescription-cost").value = "";
    const customName = row.querySelector(".custom-medicine-name");
    const customCost = row.querySelector(".custom-medicine-cost");
    if (customName) {
      customName.value = "";
      customName.setCustomValidity("");
    }
    if (customCost) {
      customCost.value = "";
      customCost.setCustomValidity("");
    }
    prescriptionRows.appendChild(row);
    syncPrescriptionRows();
  }

  addPrescriptionRow?.addEventListener("click", addMedicineRow);
  prescriptionRows?.addEventListener("change", (event) => {
    if (event.target.matches(".prescription-select, .custom-medicine-cost")) syncPrescriptionRows();
  });
  prescriptionRows?.addEventListener("input", (event) => {
    if (event.target.matches(".custom-medicine-name, .custom-medicine-cost")) syncPrescriptionRows();
  });
  prescriptionRows?.addEventListener("click", (event) => {
    if (!event.target.matches(".remove-prescription-row")) return;
    event.target.closest(".prescription-row")?.remove();
    syncPrescriptionRows();
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
  syncPrescriptionRows();
  render();
})();
