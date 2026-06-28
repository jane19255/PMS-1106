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
        firebaseUid: document.getElementById(`${prefix}-firebaseUid`)?.value.trim() || "",
        firstName: document.getElementById(`${prefix}-firstName`)?.value.trim() || "",
        lastName: document.getElementById(`${prefix}-lastName`)?.value.trim() || "",
        role: document.getElementById(`${prefix}-role`)?.value || "Receptionist",
        phone: document.getElementById(`${prefix}-phone`)?.value.trim() || "",
        email: document.getElementById(`${prefix}-email`)?.value.trim() || "",
        status: document.getElementById(`${prefix}-status`)?.value || "Active",
    };
}

async function saveNewStaff(button) {
    if (!verifyInput(button)) return;

    try {
        const response = await fetch("/api/staff", {
            method: "POST",
            headers: { "Content-Type": "application/json", Accept: "application/json" },
            body: JSON.stringify(staffPayload("add")),
        });
        if (!response.ok) throw new Error(await response.text());
        const staff = emptyStaffDefaults(await response.json());
        staffs.push(staff);
        refreshStaffList();
        notify("New staff added!", "success");
        closeModal(button);
        clearInput(button);
    } catch (error) {
        notify(error.message || "Staff could not be saved", "error");
    }
}

async function saveStaffChanges(button) {
    if (!verifyInput(button)) return;

    const staffId = document.getElementById("edit-id").value;
    try {
        const response = await fetch(`/api/staff/${encodeURIComponent(staffId)}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json", Accept: "application/json" },
            body: JSON.stringify(staffPayload("edit")),
        });
        if (!response.ok) throw new Error(await response.text());
        const updated = emptyStaffDefaults(await response.json());
        staffs = staffs.map((staff) => staff.id === updated.id ? updated : staff);
        refreshStaffList();
        notify("Changes saved!", "success");
        closeModal(button);
    } catch (error) {
        notify(error.message || "Staff changes could not be saved", "error");
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

    openModal("editStaffModal");
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