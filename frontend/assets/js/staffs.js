const staffs = [
    {
        id: "STF-001",
        firstName: "Off",
        lastName: "Jumpol",
        dob: "2000-05-12",
        gender: "Male",
        role: "Receptionist",
        nric: "S1234567D",
        phone: "91234567",
        email: "off.jumpol@csc.singaporehealth.sg",
        address: "123 Bedok North Street 2, #05-67, Singapore 460123",
        status: "Active"
    },
    {
        id: "STF-002",
        firstName: "Jimmy",
        lastName: "Jitaraphol",
        dob: "1999-03-22",
        gender: "Female",
        role: "Doctor",
        nric: "S7654321F",
        phone: "92345678",
        email: "gun.atthaphan@csc.singaporehealth.sg",
        address: "456 Jurong West Ave 1, #10-88, Singapore 640456",
        status: "Inactive"
    },
    {
        id: "STF-003",
        firstName: "Junior",
        lastName: "Panuwat",
        dob: "2002-07-10",
        gender: "Male",
        role: "Receptionist",
        nric: "S1122334H",
        phone: "93456789",
        email: "junior.panuwat@csc.singaporehealth.sg",
        address: "789 Tampines St 3, #03-12, Singapore 500789",
        status: "Active"
    },
    {
        id: "STF-004",
        firstName: "Mark",
        lastName: "Siwat",
        dob: "2001-02-05",
        gender: "Female",
        role: "Receptionist",
        nric: "S4433221Z",
        phone: "94567890",
        email: "mark.siwat@csc.singaporehealth.sg",
        address: "11 Woodlands Dr 50, #07-23, Singapore 730811",
        status: "Active"
    },
    {
        id: "STF-005",
        firstName: "William",
        lastName: "Jakrapatr",
        dob: "2003-09-18",
        gender: "Male",
        role: "Doctor",
        nric: "S5566778D",
        phone: "95678901",
        email: "william.jakrapatr@csc.singaporehealth.sg",
        address: "22 Simei St 4, #09-45, Singapore 520922",
        status: "Active"
    }
];

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

function renderStaffRow(staff, index) {
    return `
    <tr class="hover:bg-slate-50">
        <td>${staff.id}</td>
        <td>${staff.firstName + " " + staff.lastName}</td>
        <td>${staff.gender}</td>
        <td>${staff.role}</td>
        <td>${formatDate(staff.dob)}</td>
        <td>${staff.phone}</td>
        <td><span class="status ${staff.status.toLowerCase()}">${staff.status}</span></td>
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

function viewStaff(index) {
    const s = pagination.paginatedData[index];

    document.getElementById("view-pid").innerText = s.id;
    document.getElementById("view-fullname").innerText = s.firstName + " " + s.lastName;
    document.getElementById("view-gender").innerText = s.gender;
    document.getElementById("view-dob").innerText = formatDate(s.dob);
    document.getElementById("view-nric").innerText = s.nric;
    document.getElementById("view-role").innerText = s.role;

    document.getElementById("view-phone").innerText = s.phone;
    document.getElementById("view-email").innerText = s.email;
    document.getElementById("view-address").innerText = s.address;

    const statusEl = document.getElementById("view-status");
    statusEl.innerText = s.status;
    statusEl.className = "status " + s.status.toLowerCase();

    openModal("detailsModal");
}

function editStaff(index) {
    const s = pagination.paginatedData[index];

    const fullIndex = staffs.findIndex(x => x.id === s.id);
    document.getElementById("edit-index").value = fullIndex;

    document.getElementById("edit-firstName").value = s.firstName;
    document.getElementById("edit-lastName").value = s.lastName;
    document.getElementById("edit-dob").value = s.dob;
    document.getElementById("edit-gender").value = s.gender.toLowerCase();
    document.getElementById("edit-nric").value = s.nric;
    document.getElementById("edit-role").value = s.role;
    document.getElementById("edit-phone").value = s.phone;
    document.getElementById("edit-email").value = s.email;
    document.getElementById("edit-address").value = s.address;

    openModal("editStaffModal");
}

function applySearch() {
    const keyword = document.getElementById("searchInput")?.value.toLowerCase() || "";
    return staffs.filter(s =>
        s.id.toLowerCase().includes(keyword) ||
        s.firstName.toLowerCase().includes(keyword) ||
        s.lastName.toLowerCase().includes(keyword) ||
        s.phone.includes(keyword)
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
        if (sortBy === "dob") return new Date(a.dob) - new Date(b.dob);
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
    // Calendar disable future date
    const today = new Date().toLocaleDateString('en-CA');
    const dateInputs = document.querySelectorAll('input[type="date"]');

    dateInputs.forEach(input => {
        input.setAttribute('max', today);
    });

    refreshStaffList();
});