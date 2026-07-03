let medicalRecords = [];
let activePatientFilter = "";
let allPatients = [];
let allDoctors = [];

function escapeHtml(value) {
    return String(value ?? "").replace(/[&<>"']/g, (char) => ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
    })[char]);
}

function formatDateTime(dateString) {
    if (!dateString) return "N.A.";
    const date = new Date(dateString);
    if (isNaN(date)) return dateString;
    const datePart = date.toLocaleDateString("en-GB", { day: "2-digit", month: "short", year: "numeric" });
    const timePart = date.toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" });
    return `${datePart} ${timePart}`;
}

const pagination = new Pagination({
    data: [],
    rowsPerPage: 5,
    tbodyId: "medicalRecordsTableBody",
    pageInfoId: "pageInfo",
    pageSelectId: "pageSelect",
    prevBtnId: "prevBtn",
    nextBtnId: "nextBtn",
    renderRow: renderMedicalRecordRow
});

function renderMedicalRecordRow(record) {
    const diagnosis = record.diagnosis
        ? escapeHtml(record.diagnosis)
        : '<span class="tag">&mdash;</span>';
    const doctorId = record.doctor_id
        ? escapeHtml(record.doctor_id)
        : '<span class="tag">&mdash;</span>';

    return `<tr>
        <td><code style="font-size:12px">${escapeHtml(record.id)}</code></td>
        <td>
            <a href="/patients/${encodeURIComponent(record.patient_id)}/timeline" style="color:var(--primary);font-weight:500">
                ${escapeHtml(record.patient_id)}
            </a>
        </td>
        <td>${diagnosis}</td>
        <td>${formatDateTime(record.recorded_at)}</td>
        <td>${doctorId}</td>
        <td class="action">
            <div class="has-tooltip">
                <i class="view fa-solid fa-circle-info" onclick="window.location.href='/medical-records/${encodeURIComponent(record.id)}'"></i>
                <span class="tooltip-text">View Record</span>
            </div>
        </td>
    </tr>`;
}

function applySearch(records) {
    const keyword = document.getElementById("searchInput").value.trim().toLowerCase();
    if (!keyword) return records;
    return records.filter(record =>
        [record.id, record.patient_id, record.diagnosis, record.doctor_id]
            .some(field => (field || "").toLowerCase().includes(keyword))
    );
}

function applySort(records) {
    const sortBy = document.getElementById("sortBy").value;
    const sorted = [...records];
    if (sortBy === "recorded_asc") {
        sorted.sort((a, b) => new Date(a.recorded_at) - new Date(b.recorded_at));
    } else if (sortBy === "patient_id") {
        sorted.sort((a, b) => (a.patient_id || "").localeCompare(b.patient_id || ""));
    } else {
        sorted.sort((a, b) => new Date(b.recorded_at) - new Date(a.recorded_at));
    }
    return sorted;
}

function refreshMedicalRecordsList() {
    let result = applySearch(medicalRecords);
    result = applySort(result);

    pagination.data = result;
    pagination.currentPage = 1;
    pagination.renderTable();

    if (result.length === 0) {
        const message = activePatientFilter
            ? `No records for patient ID <strong>${escapeHtml(activePatientFilter)}</strong>.`
            : "No medical records found.";
        document.getElementById("medicalRecordsTableBody").innerHTML =
            `<tr><td class="empty-row" colspan="6">${message}</td></tr>`;
    }
}

async function ensurePatientAndDoctorOptions() {
    if (allPatients.length === 0) {
        try {
            const response = await fetch("/api/patients", { headers: { Accept: "application/json" } });
            if (response.ok) allPatients = await response.json();
        } catch (error) {
            console.warn("Could not load patients:", error);
        }
    }
    if (allDoctors.length === 0) {
        try {
            const response = await fetch("/api/doctors", { headers: { Accept: "application/json" } });
            if (response.ok) allDoctors = await response.json();
        } catch (error) {
            console.warn("Could not load doctors:", error);
        }
    }

    const patientSelect = document.getElementById("add-record-patient");
    patientSelect.innerHTML = '<option value="" disabled selected>Select a patient&hellip;</option>' +
        allPatients
            .map(p => `<option value="${escapeHtml(p.id)}">${escapeHtml(p.firstName)} ${escapeHtml(p.lastName)} — ${escapeHtml(p.nric)}</option>`)
            .join("");

    const doctorSelect = document.getElementById("add-record-doctor");
    doctorSelect.innerHTML = '<option value="">Not assigned</option>' +
        allDoctors
            .map(d => `<option value="${escapeHtml(d.id)}">${escapeHtml(d.name)} — ${escapeHtml(d.specialization)}</option>`)
            .join("");

    if (activePatientFilter && allPatients.some(p => p.id === activePatientFilter)) {
        patientSelect.value = activePatientFilter;
    }
}

async function openNewRecordModal() {
    await ensurePatientAndDoctorOptions();
    openModal("addRecordModal");
}

async function saveNewMedicalRecord(button) {
    const modal = button.closest(".modal");
    const errorBox = modal.querySelector(".error-box");
    errorBox.style.display = "none";

    const patientId = document.getElementById("add-record-patient").value;
    if (!patientId) {
        errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i> Select a patient.';
        errorBox.style.display = "flex";
        return;
    }

    const payload = {
        patient_id: patientId,
        doctor_id: document.getElementById("add-record-doctor").value || null,
        blood_pressure: document.getElementById("add-record-bp").value || null,
        temperature: document.getElementById("add-record-temp").value || null,
        pulse_rate: document.getElementById("add-record-pulse").value || null,
        height_cm: document.getElementById("add-record-height").value || null,
        weight_kg: document.getElementById("add-record-weight").value || null,
        reason_of_visit: document.getElementById("add-record-reason").value || null,
        clinical_findings: document.getElementById("add-record-findings").value || null,
        diagnosis: document.getElementById("add-record-diagnosis").value || null,
        treatment_plan: document.getElementById("add-record-treatment").value || null,
        doctor_notes: document.getElementById("add-record-notes").value || null,
    };

    try {
        const response = await fetch("/api/medical-records", {
            method: "POST",
            headers: { "Content-Type": "application/json", Accept: "application/json" },
            body: JSON.stringify(payload)
        });
        if (!response.ok) throw new Error(await response.text());

        const record = await response.json();
        medicalRecords.push(record);
        refreshMedicalRecordsList();
        if (typeof showToast === "function") showToast("Medical record created", "success");
        closeModal(button);
        clearInput(button);
    } catch (error) {
        errorBox.innerHTML = `<i class="fa-solid fa-circle-exclamation"></i> ${escapeHtml(error.message || "Could not save the record.")}`;
        errorBox.style.display = "flex";
    }
}

async function loadMedicalRecords() {
    try {
        const response = await fetch("/api/medical-records", { headers: { Accept: "application/json" } });
        if (!response.ok) throw new Error(await response.text());
        medicalRecords = await response.json();
        refreshMedicalRecordsList();
    } catch (error) {
        console.warn("Could not load medical records:", error);
        medicalRecords = [];
        document.getElementById("medicalRecordsTableBody").innerHTML =
            `<tr><td class="empty-row" colspan="6">${escapeHtml(error.message || "Could not load medical records.")}</td></tr>`;
        if (typeof showToast === "function") {
            showToast("Could not load medical records", "danger");
        }
    }
}

document.addEventListener("DOMContentLoaded", () => {
    const params = new URLSearchParams(window.location.search);
    const patientId = params.get("patient_id");
    if (patientId) {
        activePatientFilter = patientId;
        document.getElementById("searchInput").value = patientId;
    }

    loadMedicalRecords();
});
