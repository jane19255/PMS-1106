let appointmentsData = [];
let doctorsData = [];
let patientsData = [];
let currentEditId = null;

const TIME_SLOTS = [
    "08:00", "08:30", "09:00", "09:30", "10:00", "10:30", "11:00", "11:30",
    "13:00", "13:30", "14:00", "14:30", "15:00", "15:30", "16:00", "16:30",
    "17:00", "17:30", "18:00", "18:30",
];

const pagination = new Pagination({
    data: appointmentsData,
    rowsPerPage: 5,
    tbodyId: "appointmentTableBody",
    pageInfoId: "pageInfo",
    pageSelectId: "pageSelect",
    prevBtnId: "prevBtn",
    nextBtnId: "nextBtn",
    renderRow: renderAppointmentRow
});

function doctorById(id) {
    return doctorsData.find(d => d.id === id);
}

function patientById(id) {
    return patientsData.find(p => p.id === id);
}

function doctorLabel(id) {
    const doctor = doctorById(id);
    return doctor ? doctor.name : id;
}

function patientLabel(id) {
    const patient = patientById(id);
    return patient ? `${patient.firstName} ${patient.lastName}` : id;
}

function splitScheduledAt(iso) {
    // scheduled_at is stored as literal clock digits tagged UTC (no real timezone
    // math elsewhere in this app), so read it back with the UTC getters.
    const dt = new Date(iso);
    const date = iso.split("T")[0];
    let hours = dt.getUTCHours();
    const minutes = String(dt.getUTCMinutes()).padStart(2, "0");
    const ampm = hours >= 12 ? "PM" : "AM";
    hours = hours % 12 || 12;
    return { date, time24: `${String(dt.getUTCHours()).padStart(2, "0")}:${minutes}`, timeDisplay: `${hours}:${minutes} ${ampm}` };
}

function renderAppointmentRow(item) {
    const { date, timeDisplay } = splitScheduledAt(item.scheduled_at);
    const statusClass = (item.status || "").toLowerCase().replace(/\s+/g, '');
    const priorityClass = (item.priority || "Normal").toLowerCase().replace(/-/g, '');
    return `
    <tr>
        <td>${item.id}</td>
        <td>${patientLabel(item.patient_id)}</td>
        <td>${doctorLabel(item.doctor_id)}</td>
        <td>${date}</td>
        <td>${timeDisplay}</td>
        <td>${item.appointment_type || '—'}</td>
        <td><span class="status ${statusClass}">${item.status}</span></td>
        <td><span class="priority ${priorityClass}">${item.priority || 'Normal'}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <i class="view fa-solid fa-circle-info" onclick="viewAppointment('${item.id}')"></i>
                <span class="tooltip-text">View Details</span>
            </div>
            <div class="has-tooltip">
                <i class="edit fa-solid fa-pen-to-square" onclick="editAppointment('${item.id}')"></i>
                <span class="tooltip-text">Edit Details</span>
            </div>
        </td>
    </tr>
  `;
}

function parseDate(dateStr) {
    return new Date(dateStr);
}

function filterByDateRange(list, dateRange) {
    if (!dateRange || dateRange === "") return list;

    const today = new Date();
    today.setHours(0, 0, 0, 0);

    const oneWeekLater = new Date(today);
    oneWeekLater.setDate(today.getDate() + 7);

    return list.filter(item => {
        const apptDate = parseDate(splitScheduledAt(item.scheduled_at).date);
        apptDate.setHours(0, 0, 0, 0);

        switch (dateRange) {
            case "today":
                return apptDate.getTime() === today.getTime();
            case "thisweek":
                return apptDate >= today && apptDate < oneWeekLater;
            case "thismonth":
                return apptDate.getMonth() === today.getMonth() && apptDate.getFullYear() === today.getFullYear();
            default:
                return true;
        }
    });
}

function refreshAppointmentList() {
    let list = [...appointmentsData];

    const keyword = document.getElementById("searchInput")?.value.toLowerCase() || "";
    if (keyword) {
        list = list.filter(a =>
            a.id.toLowerCase().includes(keyword) ||
            patientLabel(a.patient_id).toLowerCase().includes(keyword) ||
            doctorLabel(a.doctor_id).toLowerCase().includes(keyword) ||
            (a.appointment_type || "").toLowerCase().includes(keyword)
        );
    }

    const status = document.getElementById("filter-status")?.value;
    if (status) list = list.filter(a => a.status === status);

    const types = Array.from(document.querySelectorAll(".filter-type:checked")).map(c => c.value);
    if (types.length > 0) list = list.filter(a => types.includes(a.appointment_type));

    const doctors = Array.from(document.querySelectorAll(".filter-doctor:checked")).map(c => c.value);
    if (doctors.length > 0) list = list.filter(a => doctors.includes(a.doctor_id));

    const dateRange = document.getElementById("filter-date")?.value;
    list = filterByDateRange(list, dateRange);

    const sort = document.getElementById("sortBy").value;
    list.sort((a, b) => {
        if (sort === "dateNewest") return new Date(b.scheduled_at) - new Date(a.scheduled_at);
        if (sort === "dateOldest") return new Date(a.scheduled_at) - new Date(b.scheduled_at);
        if (sort === "time") return splitScheduledAt(a.scheduled_at).time24.localeCompare(splitScheduledAt(b.scheduled_at).time24);
        if (sort === "status") return (a.status || "").localeCompare(b.status || "");
        if (sort === "doctor") return doctorLabel(a.doctor_id).localeCompare(doctorLabel(b.doctor_id));
        return 0;
    });

    pagination.data = list;
    pagination.currentPage = 1;
    pagination.renderTable();
}

function updateStatCards() {
    const todayStr = new Date().toISOString().split("T")[0];
    const now = new Date();

    const todayCount = appointmentsData.filter(a => splitScheduledAt(a.scheduled_at).date === todayStr).length;
    const upcomingCount = appointmentsData.filter(a =>
        new Date(a.scheduled_at) > now &&
        !["Completed", "Cancelled", "No Show"].includes(a.status)
    ).length;
    const completedThisMonth = appointmentsData.filter(a => {
        if (a.status !== "Completed") return false;
        const d = new Date(a.scheduled_at);
        return d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear();
    }).length;
    const cancelledCount = appointmentsData.filter(a => a.status === "Cancelled").length;

    document.querySelector("#todayAppointments .number").textContent = todayCount;
    document.querySelector("#upcomingAppointments .number").textContent = upcomingCount;
    document.querySelector("#completedAppointments .number").textContent = completedThisMonth;
    document.querySelector("#cancelledAppointments .number").textContent = cancelledCount;
}

async function loadAppointments() {
    try {
        const res = await fetch("/api/appointments");
        if (!res.ok) throw new Error(`Appointments API returned ${res.status}`);
        appointmentsData = await res.json();
    } catch (error) {
        console.error("Could not load appointments:", error);
        appointmentsData = [];
        showToast("Could not load appointments", "error");
    }
    updateStatCards();
    refreshAppointmentList();
}

async function loadPatients() {
    try {
        const res = await fetch("/api/patients");
        if (!res.ok) throw new Error(`Patients API returned ${res.status}`);
        patientsData = await res.json();
    } catch (error) {
        console.error("Could not load patients:", error);
        patientsData = [];
    }

    const options = patientsData
        .map(p => `<option value="${p.id}">${p.firstName} ${p.lastName} (${p.id})</option>`)
        .join("");

    const addSelect = document.getElementById("patientsList");
    if (addSelect) addSelect.innerHTML = options;

    const editSelect = document.getElementById("editPatientsList");
    if (editSelect) editSelect.innerHTML = options;
}

async function loadDoctors() {
    try {
        const res = await fetch("/api/doctors");
        if (!res.ok) throw new Error(`Doctor API returned ${res.status}`);
        doctorsData = await res.json();
    } catch (error) {
        console.warn("Could not load doctors from backend:", error);
        doctorsData = [];
    }

    const bookableDoctors = doctorsData.filter(d => d.status === "Available");

    const addSelect = document.getElementById("doctorList");
    if (addSelect) {
        if (bookableDoctors.length === 0) {
            addSelect.innerHTML = `<option value="">No available doctors</option>`;
            addSelect.disabled = true;
        } else {
            addSelect.disabled = false;
            addSelect.innerHTML = bookableDoctors
                .map(d => `<option value="${d.id}">${d.name} — ${d.specialization}</option>`)
                .join("");
        }
    }

    const editSelect = document.getElementById("editDoctorList");
    if (editSelect) {
        editSelect.innerHTML = doctorsData
            .map(d => {
                const statusLabel = d.status !== "Available" ? ` (${d.status})` : "";
                return `<option value="${d.id}">${d.name} — ${d.specialization}${statusLabel}</option>`;
            })
            .join("");
    }

    const checkbox = document.getElementById("doctorCheckbox");
    if (checkbox) {
        checkbox.innerHTML = doctorsData
            .map(d => `<div><input type="checkbox" class="filter-doctor" id="doc-${d.id}" name="doc-${d.id}" value="${d.id}">
                            <label for="doc-${d.id}">${d.name}</label>
                        </div>`)
            .join("");
    }
}

function getSelectedPriority(modal) {
    const selected = modal.querySelector(".level > div.selected");
    if (!selected) return "Normal";
    return selected.textContent.trim();
}

function setSelectedPriority(modal, priority) {
    modal.querySelectorAll(".level > div").forEach(level => {
        level.classList.toggle("selected", level.textContent.trim().toLowerCase() === (priority || "Normal").toLowerCase());
    });
}

function getSelectedSlot(modal) {
    const selected = modal.querySelector(".timeslot .slot.selected");
    return selected ? selected.dataset.time : null;
}

function setSelectedSlot(modal, time24) {
    modal.querySelectorAll(".timeslot .slot").forEach(slot => {
        slot.classList.toggle("selected", slot.dataset.time === time24);
    });
    const target = modal.querySelector(`.timeslot .slot[data-time="${time24}"]`);
    if (target) {
        const tabContent = target.closest(".tab-content");
        const tabId = tabContent?.id;
        const tabButton = modal.querySelector(`.tab[onclick*="${tabId}"]`);
        if (tabButton) openTab(tabButton, tabId);
    }
}

function computeBookedRanges(doctorId, excludeId) {
    return appointmentsData
        .filter(a => a.doctor_id === doctorId && a.id !== excludeId && !["Cancelled", "No Show"].includes(a.status))
        .map(a => {
            const start = new Date(a.scheduled_at);
            const end = new Date(start.getTime() + (a.duration_minutes || 30) * 60000);
            return { start, end };
        });
}

function refreshSlotAvailability(modal, dateInputId, doctorSelectId, excludeId) {
    const dateValue = document.getElementById(dateInputId)?.value;
    const doctorId = document.getElementById(doctorSelectId)?.value;
    if (!dateValue || !doctorId) return;

    const booked = computeBookedRanges(doctorId, excludeId);

    modal.querySelectorAll(".timeslot .slot").forEach(slot => {
        const time = slot.dataset.time;
        const slotStart = new Date(`${dateValue}T${time}:00Z`);
        const isBooked = booked.some(range => slotStart >= range.start && slotStart < range.end);
        slot.classList.toggle("disable", isBooked);
        if (isBooked) slot.classList.remove("selected");
    });
}

function viewAppointment(id) {
    const item = appointmentsData.find(a => a.id === id);
    if (!item) return;
    openModal("detailsModal");
    fillViewData(item);
}

function fillViewData(item) {
    const { date, timeDisplay } = splitScheduledAt(item.scheduled_at);
    document.getElementById("view-doc").innerText = doctorLabel(item.doctor_id);
    document.getElementById("view-date").innerText = date;
    document.getElementById("view-time").innerText = timeDisplay;
    document.getElementById("view-type").innerText = item.appointment_type || "—";
    document.getElementById("view-room").innerText = item.room || "—";
    document.getElementById("view-patient").innerText = patientLabel(item.patient_id);
    const sr = item.special_requirements || [];
    document.getElementById("view-sr").innerText = sr.length > 0 ? sr.join(", ") : "None";
    document.getElementById("view-rp").innerText = item.referring_provider || "None";
}

function editAppointment(id) {
    const item = appointmentsData.find(a => a.id === id);
    if (!item) return;
    currentEditId = id;
    openModal("editModal");
    fillEditData(item);
}

function fillEditData(item) {
    const modal = document.getElementById("editModal");
    const { date, time24 } = splitScheduledAt(item.scheduled_at);

    document.getElementById("editDoctorList").value = item.doctor_id;
    document.getElementById("edit_appointment_date").value = date;
    if (window.editDatePicker) window.editDatePicker.setDate(date, false);
    document.getElementById("editAppointmentRoom").value = item.room || "Room 1";
    document.getElementById("editAppointmentType").value = item.appointment_type || "Routine Checkup";
    document.getElementById("editPatientsList").value = item.patient_id;
    document.getElementById("editReferringProvider").value = item.referring_provider || "";

    const sr = item.special_requirements || [];
    document.getElementById("editWheelchair").checked = sr.includes("wheelchair");
    document.getElementById("editEquipment").checked = sr.includes("medical equipment");
    document.getElementById("editTranslator").checked = sr.includes("translator");

    setSelectedPriority(modal, item.priority);
    refreshSlotAvailability(modal, "edit_appointment_date", "editDoctorList", item.id);
    setSelectedSlot(modal, time24);
}

function showAddPatientTab(cb) {
    const tab = document.getElementById("addNewPatientTab");
    const inputs = tab.querySelectorAll('input, textarea, select');
    tab.classList.toggle('show');

    inputs.forEach(input => {
        if (!cb.checked) {
            if (input.type === 'checkbox' || input.type === 'radio') {
                input.checked = false;
            } else {
                input.value = '';
            }
            input.required = false;
        } else {
            input.required = true;
        }
    });

    const patientSelect = document.getElementById("patientsList");
    if (patientSelect) patientSelect.required = !cb.checked;
}

function goToStep(modal, targetStep) {
    const maxSteps = 3;

    modal.querySelectorAll('.step-content').forEach(content => {
        content.classList.remove('active');
    });
    modal.querySelectorAll('.step-item').forEach(item => {
        item.classList.remove('active');
    });

    modal.querySelectorAll('.step-item').forEach(item => {
        const stepNum = parseInt(item.dataset.step);
        if (stepNum < targetStep) {
            item.classList.add('completed');
        }
    });

    modal.querySelector(`.step-content[data-step="${targetStep}"]`).classList.add('active');
    modal.querySelector(`.step-item[data-step="${targetStep}"]`).classList.add('active');

    const backBtn = modal.querySelector('.btn-back');
    backBtn.style.display = targetStep === 1 ? 'none' : 'block';

    const nextBtn = modal.querySelector('.btn-next');
    const modalId = modal.id;

    if (targetStep === maxSteps) {
        fillSummary(modal);
        nextBtn.innerHTML = '<i class="fa-solid fa-floppy-disk"></i>Submit';
        nextBtn.onclick = modalId === 'editModal'
            ? () => submitEditAppointment(nextBtn)
            : () => submitNewAppointment(nextBtn);
    } else {
        nextBtn.textContent = "Next";
        nextBtn.onclick = () => { if (validateStep(modal, targetStep)) nextStep(nextBtn); };
    }
}

function validateStep(modal, step) {
    const selector = `${modal.id === 'editModal' ? '#editModal' : '#addAppointmentModal'} .step-content[data-step="${step}"]`;
    if (!verifyInput(selector)) return false;

    if (step === 1) {
        const selected = modal.querySelector('.step-content[data-step="1"] .timeslot .slot.selected');
        if (!selected) {
            const errorBox = modal.querySelector('.step-content[data-step="1"] .error-box');
            errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i> Please select a time slot.';
            errorBox.style.display = 'flex';
            return false;
        }
    }

    if (step === 2 && modal.id === 'addAppointmentModal') {
        const addingNew = document.getElementById('addNewPatient').checked;
        const patientSelect = document.getElementById('patientsList');
        if (!addingNew && !patientSelect.value) {
            const errorBox = modal.querySelector('.step-content[data-step="2"] .error-box');
            errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i> Please select a patient or register a new one.';
            errorBox.style.display = 'flex';
            return false;
        }
    }

    return true;
}

function fillSummary(modal) {
    const isEdit = modal.id === 'editModal';
    const prefix = isEdit ? 'editSummary' : 'summary';
    const doctorId = document.getElementById(isEdit ? 'editDoctorList' : 'doctorList').value;
    const dateValue = document.getElementById(isEdit ? 'edit_appointment_date' : 'appointment_date').value;
    const time = getSelectedSlot(modal) || '—';

    let patientLabelText;
    if (!isEdit && document.getElementById('addNewPatient').checked) {
        const fn = document.getElementById('newPatientFirstName').value.trim();
        const ln = document.getElementById('newPatientLastName').value.trim();
        patientLabelText = `${fn} ${ln} (new patient)`;
    } else {
        const patientId = document.getElementById(isEdit ? 'editPatientsList' : 'patientsList').value;
        patientLabelText = patientLabel(patientId);
    }

    document.getElementById(`${prefix}-doc`).textContent = doctorLabel(doctorId);
    document.getElementById(`${prefix}-date`).textContent = dateValue;
    document.getElementById(`${prefix}-time`).textContent = time;
    document.getElementById(`${prefix}-patient`).textContent = patientLabelText;
}

function nextStep(button) {
    const modal = button.closest('.modal');
    const currentStep = parseInt(modal.querySelector('.step-item.active').dataset.step);
    const maxSteps = 3;

    if (currentStep < maxSteps) {
        goToStep(modal, currentStep + 1);
    }
}

function prevStep(button) {
    const modal = button.closest('.modal');
    const currentStep = parseInt(modal.querySelector('.step-item.active').dataset.step);

    if (currentStep > 1) {
        goToStep(modal, currentStep - 1);
    }
}

function resetStepper(modal) {
    goToStep(modal, 1);
    modal.querySelectorAll(".step-item").forEach(step => step.classList.remove("completed"));
    modal.querySelectorAll(".timeslot .slot").forEach(slot => slot.classList.remove("selected", "disable"));
    setSelectedPriority(modal, "Normal");
}

async function createNewPatientIfNeeded() {
    const addingNew = document.getElementById('addNewPatient').checked;
    if (!addingNew) {
        return document.getElementById('patientsList').value;
    }

    const nric = document.getElementById('newPatientNric').value.trim().toUpperCase();
    const payload = {
        first_name: document.getElementById('newPatientFirstName').value.trim(),
        last_name: document.getElementById('newPatientLastName').value.trim(),
        dob: document.getElementById('newPatientDob').value,
        gender: document.getElementById('newPatientGender').value,
        nric: nric,
        nationality: document.getElementById('newPatientNationality').value.trim(),
        phone: document.getElementById('newPatientPhone').value.trim(),
        email: document.getElementById('newPatientEmail').value.trim(),
    };

    const res = await fetch('/api/patients/new', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    });

    if (!res.ok) {
        const text = await res.text();
        throw new Error(text || 'Failed to register new patient');
    }

    return nric;
}

async function submitNewAppointment(button) {
    const modal = document.getElementById('addAppointmentModal');

    let patientId;
    try {
        patientId = await createNewPatientIfNeeded();
    } catch (error) {
        showToast(error.message, 'error');
        return;
    }

    const doctorId = document.getElementById('doctorList').value;
    const dateValue = document.getElementById('appointment_date').value;
    const time = getSelectedSlot(modal);
    const appointmentType = document.getElementById('appointmentType').value;
    const specialRequirements = ['wheelchair', 'equipment', 'translator']
        .map(id => document.getElementById(id))
        .filter(el => el.checked)
        .map(el => el.value);

    const payload = {
        patient_id: patientId,
        doctor_id: doctorId,
        scheduled_at: `${dateValue}T${time}:00Z`,
        duration_minutes: 30,
        reason: appointmentType,
        priority: getSelectedPriority(modal),
        room: document.getElementById('appointmentRoom').value,
        appointment_type: appointmentType,
        referring_provider: document.getElementById('referringProvider').value.trim() || null,
        special_requirements: specialRequirements,
    };

    try {
        const res = await fetch('/api/appointments', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
        });

        if (res.ok) {
            showToast('New appointment created!', 'success');
            closeModal(button);
            resetStepper(modal);
            clearInput(button);
            await loadAppointments();
        } else if (res.status === 409) {
            const data = await res.json();
            const suggestionText = (data.suggestions || [])
                .map(s => `${new Date(s.start).toLocaleString()} – ${new Date(s.end).toLocaleTimeString()}`)
                .join('; ');
            goToStep(modal, 1);
            const box = modal.querySelector('.step-content[data-step="1"] .error-box');
            box.innerHTML = `<i class="fa-solid fa-circle-exclamation"></i> ${data.message}${suggestionText ? ' Suggested: ' + suggestionText : ''}`;
            box.style.display = 'flex';
        } else {
            const text = await res.text();
            showToast(text || 'Failed to create appointment', 'error');
        }
    } catch (error) {
        showToast('Network error: ' + error.message, 'error');
    }
}

async function submitEditAppointment(button) {
    const modal = document.getElementById('editModal');
    if (!currentEditId) return;

    const doctorId = document.getElementById('editDoctorList').value;
    const dateValue = document.getElementById('edit_appointment_date').value;
    const time = getSelectedSlot(modal);
    const appointmentType = document.getElementById('editAppointmentType').value;
    const specialRequirements = ['editWheelchair', 'editEquipment', 'editTranslator']
        .map(id => document.getElementById(id))
        .filter(el => el.checked)
        .map(el => el.value);

    const payload = {
        patient_id: document.getElementById('editPatientsList').value,
        doctor_id: doctorId,
        scheduled_at: `${dateValue}T${time}:00Z`,
        duration_minutes: 30,
        reason: appointmentType,
        priority: getSelectedPriority(modal),
        room: document.getElementById('editAppointmentRoom').value,
        appointment_type: appointmentType,
        referring_provider: document.getElementById('editReferringProvider').value.trim() || null,
        special_requirements: specialRequirements,
    };

    try {
        const res = await fetch(`/api/appointments/${encodeURIComponent(currentEditId)}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
        });

        if (res.ok) {
            showToast('Changes saved!', 'success');
            closeModal(button);
            resetStepper(modal);
            await loadAppointments();
        } else if (res.status === 409) {
            const data = await res.json();
            const suggestionText = (data.suggestions || [])
                .map(s => `${new Date(s.start).toLocaleString()} – ${new Date(s.end).toLocaleTimeString()}`)
                .join('; ');
            goToStep(modal, 1);
            const box = modal.querySelector('.step-content[data-step="1"] .error-box');
            box.innerHTML = `<i class="fa-solid fa-circle-exclamation"></i> ${data.message}${suggestionText ? ' Suggested: ' + suggestionText : ''}`;
            box.style.display = 'flex';
        } else {
            const text = await res.text();
            showToast(text || 'Failed to save appointment', 'error');
        }
    } catch (error) {
        showToast('Network error: ' + error.message, 'error');
    }
}

document.addEventListener("DOMContentLoaded", async () => {
    await Promise.all([loadDoctors(), loadPatients()]);
    await loadAppointments();

    window.addDatePicker = flatpickr("#appointment_date", {
        enableTime: false,
        dateFormat: "Y-m-d",
        minDate: "today",
        allowInput: false,
        theme: "light",
        onChange: () => refreshSlotAvailability(document.getElementById('addAppointmentModal'), 'appointment_date', 'doctorList', null),
    });

    window.editDatePicker = flatpickr("#edit_appointment_date", {
        enableTime: false,
        dateFormat: "Y-m-d",
        minDate: "today",
        allowInput: false,
        theme: "light",
        onChange: () => refreshSlotAvailability(document.getElementById('editModal'), 'edit_appointment_date', 'editDoctorList', currentEditId),
    });

    document.getElementById('doctorList')?.addEventListener('change', () =>
        refreshSlotAvailability(document.getElementById('addAppointmentModal'), 'appointment_date', 'doctorList', null));
    document.getElementById('editDoctorList')?.addEventListener('change', () =>
        refreshSlotAvailability(document.getElementById('editModal'), 'edit_appointment_date', 'editDoctorList', currentEditId));

    document.querySelectorAll(".timeslot .slot").forEach(slot => {
        slot.addEventListener('click', () => {
            if (!slot.classList.contains('disable')) {
                const modal = slot.closest('.modal');
                modal.querySelectorAll('.timeslot .slot').forEach(s => s.classList.remove('selected'));
                slot.classList.add('selected');
            }
        });
    });

    document.querySelectorAll(".level > div").forEach(level => {
        level.addEventListener('click', () => {
            const levels = level.closest('.level').querySelectorAll('div');
            levels.forEach(el => el.classList.remove("selected"));
            level.classList.add("selected");
        });
    });

    // Initialize the "Next" button handlers for step 1 on first load
    goToStep(document.getElementById('addAppointmentModal'), 1);
    goToStep(document.getElementById('editModal'), 1);
});
