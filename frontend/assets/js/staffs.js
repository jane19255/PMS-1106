let staffs = [];

const pagination = new Pagination({
    data: [],
    rowsPerPage: 5,
    tbodyId: "staffTableBody",
    pageInfoId: "pageInfo",
    pageSelectId: "pageSelect",
    prevBtnId: "prevBtn",
    nextBtnId: "nextBtn",
    renderRow: renderStaffRow
});

function emptyStaffDefaults(staff) {
    return {
        dob: "",
        gender: "",
        nric: "",
        address: "",
        emergency: "",
        ...staff,
    };
}
function notify(message, type) {
    if (typeof showToast === "function") {
        showToast(message, type);
    }
}

function escapeHtml(value) {
    return String(value ?? "").replace(/[&<>"']/g, (char) => ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
    })[char]);
}
function renderStaffRow(staff, index) {
    const fullName = `${staff.firstName} ${staff.lastName}`.trim();
    const statusClass = (staff.status || "").toLowerCase();

    return `
    <tr class="hover:bg-slate-50">
        <td>${escapeHtml(staff.id)}</td>
        <td>${escapeHtml(fullName || "-")}</td>
        <td>${escapeHtml(staff.gender || "-")}</td>
        <td>${escapeHtml(staff.role)}</td>
        <td>${escapeHtml(staff.dob ? formatDate(staff.dob) : "-")}</td>
        <td>${escapeHtml(staff.phone || "-")}</td>
        <td><span class="status ${escapeHtml(statusClass)}">${escapeHtml(staff.status)}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <i class="view fa-solid fa-circle-info" onclick="viewStaff(${index})"></i>
                <span class="tooltip-text">View Details</span>
            </div>
            <div class="has-tooltip">
                <i class="edit fa-solid fa-pen-to-square" onclick="editStaff(${index})"></i>
                <span class="tooltip-text">Edit Details</span>
            </div>
            <div class="has-tooltip">
                <i class="fa-solid fa-key" onclick="openSetPasswordModal(${index})"></i>
                <span class="tooltip-text">Set Password</span>
            </div>
            <div class="has-tooltip">
                <i class="delete fa-solid fa-trash" onclick="deleteStaff('${escapeHtml(staff.id)}')"></i>
                <span class="tooltip-text">Delete Staff</span>
            </div>
        </td>
    </tr>`;
}

async function loadStaffs() {
    try {
        const response = await fetch("/api/staff", { headers: { Accept: "application/json" } });
        if (!response.ok) throw new Error(await response.text());
        staffs = (await response.json()).map(emptyStaffDefaults);
    } catch (error) {
        console.warn("Could not load staff from backend:", error);
        staffs = [];
        notify("Staff list could not be loaded", "error");
    }

    refreshStaffList();
}

function staffPayload(prefix) {
    return {
        // The backend creates the Firebase account and fills this UID for new staff.
        firebaseUid: document.getElementById(`${prefix}-firebaseUid`)?.value.trim() || "",
        firstName: document.getElementById(`${prefix}-firstName`)?.value.trim() || "",
        lastName: document.getElementById(`${prefix}-lastName`)?.value.trim() || "",
        dob: document.getElementById(`${prefix}-dob`)?.value || "",
        gender: document.getElementById(`${prefix}-gender`)?.value || "",
        nric: document.getElementById(`${prefix}-nric`)?.value.trim() || "",
        role: document.getElementById(`${prefix}-role`)?.value || "Receptionist",
        phone: document.getElementById(`${prefix}-phone`)?.value.trim() || "",
        email: document.getElementById(`${prefix}-email`)?.value.trim() || "",
        status: document.getElementById(`${prefix}-status`)?.value || "Active",
        address: document.getElementById(`${prefix}-address`)?.value.trim() || "",
        emergency: document.getElementById(`${prefix}-emergency`)?.value.trim() || "",
    };
}

async function saveNewStaff(button) {
    const isDoctor = document.getElementById("add-role").value === "Doctor";
    const doctorLicense = document.getElementById("add-doctorLicense").value.trim();
    const doctorSpecialization = document.getElementById("add-doctorSpecialization").value.trim();

    if (isDoctor) {
        if (!doctorLicense || !doctorSpecialization) {
            notify("License number and specialization are required for doctors", "error");
            return;
        }
    }

    try {
        const response = await fetch("/api/staff", {
            method: "POST",
            headers: { "Content-Type": "application/json", Accept: "application/json" },
            body: JSON.stringify(staffPayload("add")),
        });
        if (!response.ok) throw new Error(await response.text());
        const staff = emptyStaffDefaults(await response.json());

        if (isDoctor) {
            // A doctor needs a normal staff account first because the doctors table links by staff ID.
            const doctorResponse = await fetch("/api/doctors", {
                method: "POST",
                headers: { "Content-Type": "application/json", Accept: "application/json" },
                body: JSON.stringify({
                    staff_id: staff.id,
                    license_number: doctorLicense,
                    name: `${staff.firstName} ${staff.lastName}`.trim(),
                    specialization: doctorSpecialization,
                    contact_number: staff.phone,
                    email: staff.email,
                }),
            });
            if (!doctorResponse.ok) {
                staffs.push(staff);
                refreshStaffList();
                throw new Error(`Staff was created, but the doctor profile failed: ${await doctorResponse.text()}`);
            }
            if (typeof loadDoctors === "function") await loadDoctors();
        }

        staffs.push(staff);
        refreshStaffList();
        notify(isDoctor ? "Doctor staff and profile added!" : "New staff added!", "success");
        closeModal(button);
        clearInput(button);
    } catch (error) {
        notify(error.message || "Staff could not be saved", "error");
    }
}

function toggleAddDoctorFields() {
    // Only ask for medical details when the selected staff role is Doctor.
    const isDoctor = document.getElementById("add-role")?.value === "Doctor";
    const fields = document.getElementById("addDoctorFields");
    if (!fields) return;
    fields.hidden = !isDoctor;
    fields.querySelectorAll("input").forEach(input => {
        input.required = isDoctor;
        input.disabled = !isDoctor;
    });
}

function doctorForStaff(staffId) {
    return typeof managedDoctors === "undefined"
        ? null
        : managedDoctors.find(doctor => doctor.staff_id === staffId) || null;
}

function toggleEditDoctorFields() {
    const isDoctor = document.getElementById("edit-role")?.value === "Doctor";
    const fields = document.getElementById("editDoctorFields");
    if (!fields) return;
    fields.hidden = !isDoctor;
    fields.querySelectorAll("input").forEach(input => {
        input.required = isDoctor;
        input.disabled = !isDoctor;
    });
}

async function saveStaffChanges(button) {
    const staffId = document.getElementById("edit-id").value;
    const existingDoctor = doctorForStaff(staffId);
    const isDoctor = document.getElementById("edit-role").value === "Doctor";
    const doctorLicense = document.getElementById("edit-doctorLicense").value.trim();
    const doctorSpecialization = document.getElementById("edit-doctorSpecialization").value.trim();
    const doctorAvailability = document.getElementById("edit-doctorAvailability").value;

    if (existingDoctor && !isDoctor) {
        notify("A linked doctor must keep the Doctor role", "error");
        return;
    }
    if (isDoctor) {
        if (!doctorLicense || !doctorSpecialization) {
            notify("License number and specialization are required for doctors", "error");
            return;
        }
    }


    try {
        const response = await fetch(`/api/staff/${encodeURIComponent(staffId)}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json", Accept: "application/json" },
            body: JSON.stringify(staffPayload("edit")),
        });
        if (!response.ok) throw new Error(await response.text());
        const updated = emptyStaffDefaults(await response.json());

        if (isDoctor) {
            const doctorPayload = {
                staff_id: updated.id,
                license_number: doctorLicense,
                name: `${updated.firstName} ${updated.lastName}`.trim(),
                specialization: doctorSpecialization,
                contact_number: updated.phone,
                email: updated.email,
            };
            if (existingDoctor) {
                doctorPayload.status = doctorAvailability;
            }

            const doctorResponse = await fetch(
                existingDoctor ? `/api/doctors/${encodeURIComponent(existingDoctor.id)}` : "/api/doctors",
                {
                    method: existingDoctor ? "PUT" : "POST",
                    headers: { "Content-Type": "application/json", Accept: "application/json" },
                    body: JSON.stringify(doctorPayload),
                }
            );
            if (!doctorResponse.ok) {
                throw new Error(`Staff details were saved, but doctor details failed: ${await doctorResponse.text()}`);
            }
            if (typeof loadDoctors === "function") await loadDoctors();
        }

        staffs = staffs.map((staff) => staff.id === updated.id ? updated : staff);
        refreshStaffList();
        notify("Changes saved!", "success");
        closeModal(button);
        return true;
    } catch (error) {
        notify(error.message || "Staff changes could not be saved", "error");
    }
}

async function deleteStaff(staffId) {
    const staff = staffs.find(item => item.id === staffId);
    const name = staff ? `${staff.firstName} ${staff.lastName}`.trim() : staffId;
    const warning = staff?.role === "Doctor"
        ? `Delete ${name}, their doctor profile, schedules, and login account?`
        : `Delete ${name} and their login account?`;
    if (!confirm(`${warning} This action cannot be undone.`)) return;

    try {
        const response = await fetch(`/api/staff/${encodeURIComponent(staffId)}`, {
            method: "DELETE",
            headers: { Accept: "application/json" },
        });
        if (!response.ok) throw new Error(await response.text());
        staffs = staffs.filter(item => item.id !== staffId);
        refreshStaffList();
        notify("Staff member deleted", "success");
    } catch (error) {
        notify(error.message || "Staff member could not be deleted", "error");
    }
}

function viewStaff(index) {
    const s = pagination.paginatedData[index];

    document.getElementById("view-pid").innerText = s.id;
    document.getElementById("view-fullname").innerText = `${s.firstName} ${s.lastName}`;
    document.getElementById("view-gender").innerText = s.gender || "-";
    document.getElementById("view-dob").innerText = s.dob ? formatDate(s.dob) : "-";
    document.getElementById("view-nric").innerText = s.nric || "-";
    document.getElementById("view-role").innerText = s.role;

    document.getElementById("view-phone").innerText = s.phone || "-";
    document.getElementById("view-email").innerText = s.email;
    document.getElementById("view-address").innerText = s.address || "-";
    document.getElementById("view-emergency").innerText = s.emergency || "-";

    const statusEl = document.getElementById("view-status");
    statusEl.innerText = s.status;
    statusEl.className = "status " + s.status.toLowerCase();

    openModal("detailsModal");
}

function editStaff(index) {
    const s = pagination.paginatedData[index];

    document.getElementById("edit-id").value = s.id;
    document.getElementById("edit-firebaseUid").value = s.firebaseUid || "";
    document.getElementById("edit-firstName").value = s.firstName;
    document.getElementById("edit-lastName").value = s.lastName;
    document.getElementById("edit-dob").value = s.dob || "";
    document.getElementById("edit-gender").value = (s.gender || "male").toLowerCase();
    document.getElementById("edit-nric").value = s.nric || "";
    document.getElementById("edit-role").value = s.role;
    document.getElementById("edit-phone").value = s.phone || "";
    document.getElementById("edit-email").value = s.email;
    document.getElementById("edit-status").value = s.status;
    document.getElementById("edit-address").value = s.address || "";
    document.getElementById("edit-emergency").value = s.emergency || "";

    const doctor = doctorForStaff(s.id);
    document.getElementById("edit-doctorLicense").value = doctor?.license_number || "";
    document.getElementById("edit-doctorSpecialization").value = doctor?.specialization || "";
    document.getElementById("edit-doctorAvailability").value = doctor
        ? doctorStatusText(doctor.status)
        : "Available";
    toggleEditDoctorFields();

    openModal("editStaffModal");
}

let setPasswordStaffId = null;

function openSetPasswordModal(index) {
    const s = pagination.paginatedData[index];
    if (!s) return;

    setPasswordStaffId = s.id;
    const fullName = `${s.firstName} ${s.lastName}`.trim();
    document.getElementById("setPasswordDescription").textContent =
        `Set a new login password for ${fullName || s.id}. They can sign in with it immediately.`;
    document.getElementById("setPassword-new").value = "";
    document.getElementById("setPassword-confirm").value = "";

    const modal = document.getElementById("setPasswordModal");
    modal.querySelectorAll(".error-box").forEach(box => { box.style.display = "none"; });

    openModal("setPasswordModal");
}

async function submitSetPassword() {
    const modal = document.getElementById("setPasswordModal");
    const errorBox = modal.querySelector(".error-box");
    const newPassword = document.getElementById("setPassword-new").value;
    const confirmPassword = document.getElementById("setPassword-confirm").value;

    if (newPassword.length < 6) {
        errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i> Password must be at least 6 characters.';
        errorBox.style.display = "flex";
        return;
    }
    if (newPassword !== confirmPassword) {
        errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i> Passwords do not match.';
        errorBox.style.display = "flex";
        return;
    }

    try {
        const response = await fetch(`/api/staff/${encodeURIComponent(setPasswordStaffId)}/password`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ new_password: newPassword })
        });

        if (!response.ok) {
            const message = await response.text();
            errorBox.innerHTML = `<i class="fa-solid fa-circle-exclamation"></i> ${escapeHtml(message)}`;
            errorBox.style.display = "flex";
            return;
        }

        modal.style.display = "none";
        document.documentElement.classList.remove("no-scroll");
        notify("Password updated successfully.", "success");
    } catch (error) {
        errorBox.innerHTML = `<i class="fa-solid fa-circle-exclamation"></i> ${escapeHtml(error.message || "Could not reach the server.")}`;
        errorBox.style.display = "flex";
    }
}

function applySearch() {
    const keyword = document.getElementById("searchInput")?.value.toLowerCase() || "";
    return staffs.filter(s =>
        s.id.toLowerCase().includes(keyword) ||
        s.firstName.toLowerCase().includes(keyword) ||
        s.lastName.toLowerCase().includes(keyword) ||
        (s.phone || "").includes(keyword)
    );
}

function applyFilter(list) {
    const role = document.getElementById("filter-role")?.value || "";
    const status = document.getElementById("filter-status")?.value || "";

    return list.filter(item => {
        let match = true;
        if (role) match = match && item.role === role;
        if (status) match = match && item.status === status;
        return match;
    });
}

function applySort(list) {
    const sortBy = document.getElementById("sortBy")?.value || "id";

    const sorted = [...list];
    sorted.sort((a, b) => {
        if (sortBy === "name") return (a.firstName + a.lastName).localeCompare(b.firstName + b.lastName);
        if (sortBy === "dob") return new Date(a.dob || 0) - new Date(b.dob || 0);
        if (sortBy === "status") return a.status.localeCompare(b.status);
        return a.id.localeCompare(b.id);
    });
    return sorted;
}

function refreshStaffList() {
    let result = applySearch();
    result = applyFilter(result);
    result = applySort(result);

    pagination.data = result;
    pagination.currentPage = 1;
    pagination.renderTable();
}

document.addEventListener("DOMContentLoaded", () => {
    const today = new Date().toLocaleDateString("en-CA");
    const dateInputs = document.querySelectorAll('input[type="date"]');

    dateInputs.forEach(input => {
        input.setAttribute("max", today);
    });

    loadStaffs();
});
