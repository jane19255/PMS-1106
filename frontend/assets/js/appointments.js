let appointmentsData = [];
let doctorsData = [];
let patientsData = [];
const doctorSchedules = new Map();
let currentEditId = null;
const CLINIC_TIME_ZONE = "Asia/Singapore";
const APPOINTMENT_DURATION_MINUTES = 30;
let pendingNewPatientId = null;
const APPOINTMENT_NATIONALITIES = [
    "Singapore",
    "Malaysia",
    "Thailand",
    "Indonesia",
    "Afghan",
    "Albanian",
    "Algerian",
    "American",
    "Andorran",
    "Angolan",
    "Argentine",
    "Armenian",
    "Australian",
    "Austrian",
    "Azerbaijani",
    "Bahraini",
    "Bangladeshi",
    "Barbadian",
    "Belarusian",
    "Belgian",
    "Belizean",
    "Beninese",
    "Bhutanese",
    "Bolivian",
    "Bosnian",
    "Botswanan",
    "Brazilian",
    "British",
    "Bruneian",
    "Bulgarian",
    "Burkinabe",
    "Burmese",
    "Burundian",
    "Cambodian",
    "Cameroonian",
    "Canadian",
    "Cape Verdean",
    "Central African",
    "Chadian",
    "Chilean",
    "Chinese",
    "Colombian",
    "Comoran",
    "Congolese",
    "Costa Rican",
    "Croatian",
    "Cuban",
    "Cypriot",
    "Czech",
    "Danish",
    "Djiboutian",
    "Dominican",
    "Dutch",
    "East Timorese",
    "Ecuadorian",
    "Egyptian",
    "Emirian",
    "Equatorial Guinean",
    "Eritrean",
    "Estonian",
    "Ethiopian",
    "Fijian",
    "Filipino",
    "Finnish",
    "French",
    "Gabonese",
    "Gambian",
    "Georgian",
    "German",
    "Ghanaian",
    "Greek",
    "Grenadian",
    "Guatemalan",
    "Guinean",
    "Guyanese",
    "Haitian",
    "Honduran",
    "Hungarian",
    "Icelandic",
    "Indian",
    "Iranian",
    "Iraqi",
    "Irish",
    "Israeli",
    "Italian",
    "Ivorian",
    "Jamaican",
    "Japanese",
    "Jordanian",
    "Kazakhstani",
    "Kenyan",
    "Kuwaiti",
    "Kyrgyz",
    "Laotian",
    "Latvian",
    "Lebanese",
    "Liberian",
    "Libyan",
    "Liechtenstein",
    "Lithuanian",
    "Luxembourg",
    "Macedonian",
    "Malagasy",
    "Malawian",
    "Maldivian",
    "Maltese",
    "Marshallese",
    "Mauritanian",
    "Mauritian",
    "Mexican",
    "Micronesian",
    "Moldovan",
    "Monacan",
    "Mongolian",
    "Montenegrin",
    "Moroccan",
    "Mozambican",
    "Namibian",
    "Nepalese",
    "New Zealander",
    "Nicaraguan",
    "Nigerian",
    "Nigerien",
    "North Korean",
    "Norwegian",
    "Omani",
    "Pakistani",
    "Palauan",
    "Panamanian",
    "Paraguayan",
    "Peruvian",
    "Polish",
    "Portuguese",
    "Qatari",
    "Romanian",
    "Russian",
    "Rwandan",
    "Saint Lucian",
    "Salvadoran",
    "Samoan",
    "Sao Tomean",
    "Saudi",
    "Scottish",
    "Senegalese",
    "Serbian",
    "Seychellois",
    "Sierra Leonean",
    "Slovak",
    "Slovenian",
    "Solomon Islander",
    "Somali",
    "South African",
    "South Korean",
    "Spanish",
    "Sri Lankan",
    "Sudanese",
    "Surinamese",
    "Swazi",
    "Swedish",
    "Swiss",
    "Syrian",
    "Taiwanese",
    "Tajik",
    "Tanzanian",
    "Togolese",
    "Tongan",
    "Trinidadian",
    "Tunisian",
    "Turkish",
    "Tuvaluan",
    "Ugandan",
    "Ukrainian",
    "Uruguayan",
    "Uzbek",
    "Venezuelan",
    "Vietnamese",
    "Yemeni",
    "Zambian",
    "Zimbabwean",
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
    const dt = new Date(iso);
    const parts = new Intl.DateTimeFormat("en-CA", {
        timeZone: CLINIC_TIME_ZONE,
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        hourCycle: "h23"
    }).formatToParts(dt).reduce((result, part) => {
        result[part.type] = part.value;
        return result;
    }, {});
    const date = `${parts.year}-${parts.month}-${parts.day}`;
    const time24 = `${parts.hour}:${parts.minute}`;
    let hours = Number(parts.hour);
    const minutes = parts.minute;
    const ampm = hours >= 12 ? "PM" : "AM";
    hours = hours % 12 || 12;
    return { date, time24, timeDisplay: `${hours}:${minutes} ${ampm}` };
}

function clinicDateString(date = new Date()) {
    return new Intl.DateTimeFormat("en-CA", {
        timeZone: CLINIC_TIME_ZONE,
        year: "numeric",
        month: "2-digit",
        day: "2-digit"
    }).format(date);
}

function singaporeTimestamp(date, time) {
    return `${date}T${time}:00+08:00`;
}

function renderAppointmentRow(item) {
    const { date, timeDisplay } = splitScheduledAt(item.scheduled_at);
    const statusClass = (item.status || "").toLowerCase().replace(/\s+/g, '');
    return `
    <tr>
        <td>${item.id}</td>
        <td>${patientLabel(item.patient_id)}</td>
        <td>${doctorLabel(item.doctor_id)}</td>
        <td>${date}</td>
        <td>${timeDisplay}</td>
        <td>${item.reason || '—'}</td>
        <td><span class="status ${statusClass}">${item.status}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <i class="view fa-solid fa-circle-info" onclick="viewAppointment('${item.id}')"></i>
                <span class="tooltip-text">View Details</span>
            </div>
            <div class="has-tooltip">
                <i class="edit fa-solid fa-pen-to-square" onclick="editAppointment('${item.id}')"></i>
                <span class="tooltip-text">Edit Details</span>
            </div>
            <div class="has-tooltip">
                <i class="delete fa-solid fa-trash" onclick="deleteAppointment('${item.id}')"></i>
                <span class="tooltip-text">Cancel Appointment</span>
            </div>
        </td>
    </tr>
  `;
}

function parseDate(dateStr) {
    return new Date(`${dateStr}T00:00:00+08:00`);
}

function filterByDateRange(list, dateRange) {
    if (!dateRange || dateRange === "") return list;

    const todayString = clinicDateString();
    const today = parseDate(todayString);
    const oneWeekLater = new Date(today.getTime() + 7 * 24 * 60 * 60 * 1000);

    return list.filter(item => {
        const apptDate = parseDate(splitScheduledAt(item.scheduled_at).date);
        apptDate.setHours(0, 0, 0, 0);

        switch (dateRange) {
            case "today":
                return apptDate.getTime() === today.getTime();
            case "thisweek":
                return apptDate >= today && apptDate < oneWeekLater;
            case "thismonth":
                return splitScheduledAt(item.scheduled_at).date.slice(0, 7) === todayString.slice(0, 7);
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
            (a.reason || "").toLowerCase().includes(keyword)
        );
    }

    const status = document.getElementById("filter-status")?.value;
    if (status) list = list.filter(a => a.status === status);

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
    const todayStr = clinicDateString();
    const now = new Date();

    const todayCount = appointmentsData.filter(a => splitScheduledAt(a.scheduled_at).date === todayStr).length;
    const upcomingCount = appointmentsData.filter(a =>
        new Date(a.scheduled_at) > now &&
        !["Completed", "Cancelled", "No Show"].includes(a.status)
    ).length;
    const completedThisMonth = appointmentsData.filter(a => {
        if (a.status !== "Completed") return false;
        return splitScheduledAt(a.scheduled_at).date.slice(0, 7) === todayStr.slice(0, 7);
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

function cacheCreatedPatient(patientId, payload) {
    const cachedPatient = {
        id: patientId,
        firstName: payload.first_name,
        lastName: payload.last_name,
        dob: payload.dob,
        gender: payload.gender,
        nric: payload.nric,
        nationality: payload.nationality,
        phone: payload.phone,
        email: payload.email,
        status: "Active"
    };

    const existingIndex = patientsData.findIndex(patient => patient.id === patientId);
    if (existingIndex >= 0) {
        patientsData[existingIndex] = { ...patientsData[existingIndex], ...cachedPatient };
    } else {
        patientsData.unshift(cachedPatient);
    }
}
async function loadPatients(selectedPatientId = null) {
    const previousAddValue = selectedPatientId || document.getElementById("patientsList")?.value || "";
    const previousEditValue = document.getElementById("editPatientsList")?.value || "";
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
    if (addSelect) {
        addSelect.innerHTML = options;
        if (previousAddValue) addSelect.value = previousAddValue;
    }

    const editSelect = document.getElementById("editPatientsList");
    if (editSelect) {
        editSelect.innerHTML = options;
        if (previousEditValue) editSelect.value = previousEditValue;
    }
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

    doctorSchedules.clear();
    await Promise.all(doctorsData.map(async doctor => {
        try {
            const response = await fetch(`/api/doctors/${encodeURIComponent(doctor.id)}/schedules`);
            doctorSchedules.set(doctor.id, response.ok ? await response.json() : []);
        } catch (_error) {
            doctorSchedules.set(doctor.id, []);
        }
    }));

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
    // Ignore the appointment being edited and slots that were already released.
    return appointmentsData
        .filter(a => a.doctor_id === doctorId && a.id !== excludeId && !["Cancelled", "No Show"].includes(a.status))
        .map(a => {
            const start = new Date(a.scheduled_at);
            const end = new Date(start.getTime() + (a.duration_minutes || 30) * 60000);
            return { start, end };
        });
}

function slotFitsDoctorSchedule(doctorId, dateValue, time) {
    const schedules = doctorSchedules.get(doctorId) || [];
    const start = new Date(singaporeTimestamp(dateValue, time));
    const end = new Date(start.getTime() + APPOINTMENT_DURATION_MINUTES * 60000);
    const dayName = new Intl.DateTimeFormat("en-US", {
        timeZone: CLINIC_TIME_ZONE,
        weekday: "long"
    }).format(start);
    const endTime = splitScheduledAt(end.toISOString()).time24;

    // Use the standard clinic hours until a custom doctor schedule is created.
    if (schedules.length === 0) {
        return (time >= "08:00" && endTime <= "12:00")
            || (time >= "13:00" && endTime <= "19:00");
    }

    return schedules.some(schedule => {
        const scheduleStart = schedule.start_time.slice(0, 5);
        const scheduleEnd = schedule.end_time.slice(0, 5);
        return schedule.day_of_week === dayName && time >= scheduleStart && endTime <= scheduleEnd;
    });
}

function refreshSlotAvailability(modal, dateInputId, doctorSelectId, excludeId) {
    const dateValue = document.getElementById(dateInputId)?.value;
    const doctorId = document.getElementById(doctorSelectId)?.value;
    if (!dateValue || !doctorId) return;

    const booked = computeBookedRanges(doctorId, excludeId);

    // Disable any displayed slot that falls inside an existing booking.
    modal.querySelectorAll(".timeslot .slot").forEach(slot => {
        const time = slot.dataset.time;
        const slotStart = new Date(singaporeTimestamp(dateValue, time));
        const isBooked = booked.some(range => slotStart >= range.start && slotStart < range.end);
        const isPast = slotStart <= new Date();
        const isOutsideSchedule = !slotFitsDoctorSchedule(doctorId, dateValue, time);
        const disabled = isBooked || isPast || isOutsideSchedule;
        slot.classList.toggle("disable", disabled);
        if (disabled) slot.classList.remove("selected");
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
    document.getElementById("view-patient").innerText = patientLabel(item.patient_id);
    document.getElementById("view-reason").innerText = item.reason || "—";
    document.getElementById("view-notes").innerText = item.notes || "None";
}

function editAppointment(id) {
    const item = appointmentsData.find(a => a.id === id);
    if (!item) return;
    currentEditId = id;
    openModal("editModal");
    fillEditData(item);
}

let pendingDeleteId = null;

function deleteAppointment(id) {
    pendingDeleteId = id;
    openModal('confirmDeleteModal');
}

async function confirmDeleteAppointment(button) {
    const id = pendingDeleteId;
    if (!id) return;

    try {
        // The backend keeps the row and changes its status to Cancelled.
        const res = await fetch(`/api/appointments/${encodeURIComponent(id)}`, {
            method: 'DELETE',
        });

        if (res.ok) {
            showToast('Appointment cancelled', 'success');
            closeModal(button);
            await loadAppointments();
        } else {
            const text = await res.text();
            showToast(text || 'Failed to cancel appointment', 'error');
        }
    } catch (error) {
        showToast('Network error: ' + error.message, 'error');
    } finally {
        pendingDeleteId = null;
    }
}

function fillEditData(item) {
    const modal = document.getElementById("editModal");
    const { date, time24 } = splitScheduledAt(item.scheduled_at);

    document.getElementById("editDoctorList").value = item.doctor_id;
    document.getElementById("edit_appointment_date").value = date;
    if (window.editDatePicker) window.editDatePicker.setDate(date, false);
    document.getElementById("editPatientsList").value = item.patient_id;
    document.getElementById("editAppointmentReason").value = item.reason || "";
    document.getElementById("editAppointmentNotes").value = item.notes || "";

    refreshSlotAvailability(modal, "edit_appointment_date", "editDoctorList", item.id);
    setSelectedSlot(modal, time24);
}

function setAddPatientMode(enabled, options = {}) {
    const { clearFields = false } = options;
    const toggle = document.getElementById("addNewPatient");
    const tab = document.getElementById("addNewPatientTab");
    const patientSelect = document.getElementById("patientsList");
    const patientField = patientSelect?.closest(".field");
    if (!toggle || !tab) return;

    toggle.setAttribute("aria-pressed", String(enabled));
    toggle.classList.toggle("active", enabled);
    tab.classList.toggle("show", enabled);

    tab.querySelectorAll("input, textarea, select").forEach(input => {
        input.disabled = !enabled;
        input.required = enabled;
        if (clearFields && !enabled) {
            if (input.type === "checkbox" || input.type === "radio") {
                input.checked = false;
            } else if (input.tagName === "SELECT") {
                input.selectedIndex = 0;
            } else {
                input.value = "";
            }
        }
    });

    if (patientSelect) {
        patientSelect.disabled = enabled;
        patientSelect.required = !enabled;
        if (enabled) patientSelect.value = "";
    }

    if (patientField) {
        patientField.style.display = enabled ? "none" : "";
    }

    if (!enabled) pendingNewPatientId = null;
}

function isAddPatientMode() {
    return document.getElementById("addNewPatient")?.getAttribute("aria-pressed") === "true";
}

function showAddPatientTab(toggle) {
    const enabled = toggle.getAttribute("aria-pressed") !== "true";
    setAddPatientMode(enabled, { clearFields: !enabled });
}

function showAppointmentStepError(modal, step, message) {
    goToStep(modal, step);
    const errorBox = modal.querySelector(`.step-content[data-step="${step}"] .error-box`);
    if (errorBox) {
        errorBox.innerHTML = `<i class="fa-solid fa-circle-exclamation"></i> ${message}`;
        errorBox.style.display = "flex";
    } else {
        showToast(message, "error");
    }
}

function populateAppointmentNationalityDropdown() {
    const input = document.getElementById("newPatientNationality");
    if (!input) return;

    let list = document.getElementById("appointmentNationalityList");
    if (!list) {
        list = document.createElement("datalist");
        list.id = "appointmentNationalityList";
        input.insertAdjacentElement("afterend", list);
    }

    input.setAttribute("list", list.id);
    list.innerHTML = "";
    APPOINTMENT_NATIONALITIES.forEach(nationality => {
        const option = document.createElement("option");
        option.value = nationality;
        list.appendChild(option);
    });
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
        const addingNew = isAddPatientMode();
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
    if (!isEdit && isAddPatientMode()) {
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
    modal.querySelectorAll(".error-box").forEach(box => {
        box.innerHTML = "";
        box.style.display = "none";
    });

    if (modal.id === "addAppointmentModal") {
        setAddPatientMode(false, { clearFields: true });
    }
}

async function createNewPatientIfNeeded() {
    const addingNew = isAddPatientMode();
    if (!addingNew) {
        return document.getElementById('patientsList').value;
    }

    if (pendingNewPatientId) {
        return pendingNewPatientId;
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

    pendingNewPatientId = nric;
    cacheCreatedPatient(nric, payload);
    await loadPatients(nric);
    return nric;
}

async function submitNewAppointment(button) {
    const modal = document.getElementById('addAppointmentModal');

    let patientId;
    try {
        patientId = await createNewPatientIfNeeded();
    } catch (error) {
        showToast(error.message || 'Failed to register new patient', 'error');
        return;
    }

    const doctorId = document.getElementById('doctorList').value;
    const dateValue = document.getElementById('appointment_date').value;
    const time = getSelectedSlot(modal);

    const payload = {
        patient_id: patientId,
        doctor_id: doctorId,
        scheduled_at: singaporeTimestamp(dateValue, time),
        duration_minutes: APPOINTMENT_DURATION_MINUTES,
        reason: document.getElementById('appointmentReason').value.trim(),
        notes: document.getElementById('appointmentNotes').value.trim() || null,
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
            clearInput(button);
            resetStepper(modal);
            await loadPatients(patientId);
            await loadAppointments();
        } else if (res.status === 409) {
            const data = await res.json();
            const suggestionText = (data.suggestions || [])
                .map(s => `${new Date(s.start).toLocaleString("en-SG", { timeZone: CLINIC_TIME_ZONE })} – ${new Date(s.end).toLocaleTimeString("en-SG", { timeZone: CLINIC_TIME_ZONE })}`)
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

    const payload = {
        patient_id: document.getElementById('editPatientsList').value,
        doctor_id: doctorId,
        scheduled_at: singaporeTimestamp(dateValue, time),
        duration_minutes: APPOINTMENT_DURATION_MINUTES,
        reason: document.getElementById('editAppointmentReason').value.trim(),
        notes: document.getElementById('editAppointmentNotes').value.trim() || null,
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
                .map(s => `${new Date(s.start).toLocaleString("en-SG", { timeZone: CLINIC_TIME_ZONE })} – ${new Date(s.end).toLocaleTimeString("en-SG", { timeZone: CLINIC_TIME_ZONE })}`)
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
    populateAppointmentNationalityDropdown();
    setAddPatientMode(false, { clearFields: true });
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

    // Initialize the "Next" button handlers for step 1 on first load
    goToStep(document.getElementById('addAppointmentModal'), 1);
    goToStep(document.getElementById('editModal'), 1);

});
