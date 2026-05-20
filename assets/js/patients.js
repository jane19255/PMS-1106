const nationalities = [
    "Singapore", "Malaysia", "Thailand", "Indonesia",
    "Afghan", "Albanian", "Algerian", "American", "Andorran",
    "Angolan", "Argentine", "Armenian", "Australian", "Austrian",
    "Azerbaijani", "Bahraini", "Bangladeshi", "Barbadian", "Belarusian",
    "Belgian", "Belizean", "Beninese", "Bhutanese", "Bolivian",
    "Bosnian", "Botswanan", "Brazilian", "British", "Bruneian",
    "Bulgarian", "Burkinabe", "Burmese", "Burundian", "Cambodian",
    "Cameroonian", "Canadian", "Cape Verdean", "Central African", "Chadian",
    "Chilean", "Chinese", "Colombian", "Comoran", "Congolese",
    "Costa Rican", "Croatian", "Cuban", "Cypriot", "Czech",
    "Danish", "Djiboutian", "Dominican", "Dutch", "East Timorese",
    "Ecuadorian", "Egyptian", "Emirian", "Equatorial Guinean", "Eritrean",
    "Estonian", "Ethiopian", "Fijian", "Filipino", "Finnish",
    "French", "Gabonese", "Gambian", "Georgian", "German",
    "Ghanaian", "Greek", "Grenadian", "Guatemalan", "Guinean",
    "Guyanese", "Haitian", "Honduran", "Hungarian", "Icelandic",
    "Indian", "Iranian", "Iraqi", "Irish", "Israeli",
    "Italian", "Ivorian", "Jamaican", "Japanese", "Jordanian",
    "Kazakhstani", "Kenyan", "Kuwaiti", "Kyrgyz", "Laotian",
    "Latvian", "Lebanese", "Liberian", "Libyan", "Liechtenstein",
    "Lithuanian", "Luxembourg", "Macedonian", "Malagasy", "Malawian",
    "Maldivian", "Maltese", "Marshallese", "Mauritanian", "Mauritian",
    "Mexican", "Micronesian", "Moldovan", "Monacan", "Mongolian",
    "Montenegrin", "Moroccan", "Mozambican", "Namibian", "Nepalese",
    "New Zealander", "Nicaraguan", "Nigerian", "Nigerien", "North Korean",
    "Norwegian", "Omani", "Pakistani", "Palauan", "Panamanian",
    "Paraguayan", "Peruvian", "Polish", "Portuguese", "Qatari",
    "Romanian", "Russian", "Rwandan", "Saint Lucian", "Salvadoran",
    "Samoan", "Sao Tomean", "Saudi", "Scottish", "Senegalese",
    "Serbian", "Seychellois", "Sierra Leonean", "Slovak", "Slovenian",
    "Solomon Islander", "Somali", "South African", "South Korean", "Spanish",
    "Sri Lankan", "Sudanese", "Surinamese", "Swazi", "Swedish",
    "Swiss", "Syrian", "Taiwanese", "Tajik", "Tanzanian",
    "Togolese", "Tongan", "Trinidadian", "Tunisian", "Turkish",
    "Tuvaluan", "Ugandan", "Ukrainian", "Uruguayan", "Uzbek",
    "Venezuelan", "Vietnamese", "Yemeni", "Zambian", "Zimbabwean"
];

function populateNationalityDropdown(elementId) {
    const select = document.getElementById(elementId);
    if (!select) return;
    select.innerHTML = '';

    nationalities.forEach(nation => {
        const option = document.createElement('option');
        option.value = nation;
        option.textContent = nation;
        select.appendChild(option);
    });
}

const patients = [
    {
        id: "PAT-001",
        firstName: "Off",
        lastName: "Jumpol",
        dob: "2000-05-12",
        gender: "Male",
        nric: "S1234567D",
        nationality: "Singapore",
        phone: "91234567",
        email: "off.jumpol@csc.singaporehealth.sg",
        emergencyName: "Jane Jumpol",
        emergencyPhone: "81112222",
        address: "123 Bedok North Street 2, #05-67, Singapore 460123",
        allergies: "None",
        medications: "None",
        conditions: "Hypertension",
        status: "Active"
    },
    {
        id: "PAT-002",
        firstName: "Gun",
        lastName: "Atthaphan",
        dob: "1999-03-22",
        gender: "Female",
        nric: "S7654321F",
        nationality: "Thailand",
        phone: "92345678",
        email: "gun.atthaphan@csc.singaporehealth.sg",
        emergencyName: "Mark Siwat",
        emergencyPhone: "82223333",
        address: "456 Jurong West Ave 1, #10-88, Singapore 640456",
        allergies: "Penicillin",
        medications: "Loratadine",
        conditions: "Asthma",
        status: "Inactive"
    },
    {
        id: "PAT-003",
        firstName: "Junior",
        lastName: "Panuwat",
        dob: "2002-07-10",
        gender: "Male",
        nric: "S1122334H",
        nationality: "Malaysia",
        phone: "93456789",
        email: "junior.panuwat@csc.singaporehealth.sg",
        emergencyName: "Lisa Panuwat",
        emergencyPhone: "83334444",
        address: "789 Tampines St 3, #03-12, Singapore 500789",
        allergies: "Latex",
        medications: "None",
        conditions: "None",
        status: "Active"
    },
    {
        id: "PAT-004",
        firstName: "Mark",
        lastName: "Siwat",
        dob: "2001-02-05",
        gender: "Female",
        nric: "S4433221Z",
        nationality: "Singapore",
        phone: "94567890",
        email: "mark.siwat@csc.singaporehealth.sg",
        emergencyName: "Gun Atthaphan",
        emergencyPhone: "84445555",
        address: "11 Woodlands Dr 50, #07-23, Singapore 730811",
        allergies: "None",
        medications: "Multivitamins",
        conditions: "Diabetes Type 2",
        status: "Active"
    },
    {
        id: "PAT-005",
        firstName: "William",
        lastName: "Jakrapatr",
        dob: "2003-09-18",
        gender: "Male",
        nric: "S5566778D",
        nationality: "Singapore",
        phone: "95678901",
        email: "william.jakrapatr@csc.singaporehealth.sg",
        emergencyName: "Anna Jakrapatr",
        emergencyPhone: "85556666",
        address: "22 Simei St 4, #09-45, Singapore 520922",
        allergies: "Dust",
        medications: "None",
        conditions: "Eczema",
        status: "Active"
    },
    {
        id: "PAT-006",
        firstName: "Est",
        lastName: "Werawat",
        dob: "2004-11-30",
        gender: "Female",
        nric: "S8877665F",
        nationality: "Indonesia",
        phone: "96789012",
        email: "est.werawat@csc.singaporehealth.sg",
        emergencyName: "Peter Werawat",
        emergencyPhone: "86667777",
        address: "33 Pasir Ris Dr 12, #02-77, Singapore 510433",
        allergies: "None",
        medications: "None",
        conditions: "None",
        status: "Inactive"
    },
    {
        id: "PAT-007",
        firstName: "Sea",
        lastName: "Tawinan",
        dob: "2002-04-14",
        gender: "Female",
        nric: "S9988776H",
        nationality: "Singapore",
        phone: "97890123",
        email: "sea.tawinan@csc.singaporehealth.sg",
        emergencyName: "Chloe Tawinan",
        emergencyPhone: "87778888",
        address: "44 Hougang Ave 6, #06-32, Singapore 530044",
        allergies: "None",
        medications: "None",
        conditions: "Hypertension",
        status: "Active"
    },
    {
        id: "PAT-008",
        firstName: "Keng",
        lastName: "Harit",
        dob: "2001-01-09",
        gender: "Male",
        nric: "S2233445Z",
        nationality: "Singapore",
        phone: "98901234",
        email: "keng.harit@csc.singaporehealth.sg",
        emergencyName: "Tom Harit",
        emergencyPhone: "88889999",
        address: "55 Bukit Batok St 22, #01-56, Singapore 650055",
        allergies: "None",
        medications: "Aspirin",
        conditions: "Heart Condition",
        status: "Inactive"
    },
    {
        id: "PAT-009",
        firstName: "Namping",
        lastName: "Napasatkron",
        dob: "2003-06-07",
        gender: "Female",
        nric: "S3344556D",
        nationality: "Thailand",
        phone: "99012345",
        email: "namping.napasatkron@csc.singaporehealth.sg",
        emergencyName: "Sara Napasatkron",
        emergencyPhone: "89990000",
        address: "66 Ang Mo Kio Ave 3, #08-99, Singapore 560066",
        allergies: "Food: Seafood",
        medications: "None",
        conditions: "None",
        status: "Active"
    },
    {
        id: "PAT-010",
        firstName: "Tle",
        lastName: "Thanapon",
        dob: "2000-12-12",
        gender: "Male",
        nric: "S4455667F",
        nationality: "Singapore",
        phone: "90123456",
        email: "tle.thanapon@csc.singaporehealth.sg",
        emergencyName: "Ben Thanapon",
        emergencyPhone: "80001111",
        address: "77 Serangoon Ave 1, #04-21, Singapore 550077",
        allergies: "None",
        medications: "None",
        conditions: "None",
        status: "Active"
    },
    {
        id: "PAT-011",
        firstName: "Firstone",
        lastName: "Kanaphan",
        dob: "2004-08-19",
        gender: "Female",
        nric: "S5566778H",
        nationality: "Singapore",
        phone: "91123345",
        email: "firstone.kanaphan@csc.singaporehealth.sg",
        emergencyName: "May Kanaphan",
        emergencyPhone: "81110000",
        address: "88 Punggol Dr 10, #03-65, Singapore 820088",
        allergies: "None",
        medications: "None",
        conditions: "Asthma",
        status: "Inactive"
    },
    {
        id: "PAT-012",
        firstName: "Auau",
        lastName: "Thanaphum",
        dob: "1998-07-04",
        gender: "Male",
        nric: "S6677889Z",
        nationality: "Malaysia",
        phone: "92234456",
        email: "auau.thanaphum@csc.singaporehealth.sg",
        emergencyName: "Ken Thanaphum",
        emergencyPhone: "82221111",
        address: "99 Choa Chu Kang Ave 2, #05-11, Singapore 680099",
        allergies: "None",
        medications: "Metformin",
        conditions: "Diabetes Type 2",
        status: "Active"
    },
    {
        id: "PAT-013",
        firstName: "Save",
        lastName: "Worapong",
        dob: "2002-02-22",
        gender: "Male",
        nric: "S7788990D",
        nationality: "Singapore",
        phone: "93345567",
        email: "save.worapong@csc.singaporehealth.sg",
        emergencyName: "Jake Worapong",
        emergencyPhone: "83332222",
        address: "100 Bishan St 22, #09-34, Singapore 570100",
        allergies: "None",
        medications: "None",
        conditions: "None",
        status: "Inactive"
    },
    {
        id: "PAT-014",
        firstName: "Fluke",
        lastName: "Pusit",
        dob: "2001-10-01",
        gender: "Male",
        nric: "S8899001F",
        nationality: "Singapore",
        phone: "94456678",
        email: "fluke.pusit@csc.singaporehealth.sg",
        emergencyName: "Rita Pusit",
        emergencyPhone: "84443333",
        address: "111 Toa Payoh Lorong 1, #02-87, Singapore 310111",
        allergies: "None",
        medications: "None",
        conditions: "Hypertension",
        status: "Active"
    },
    {
        id: "PAT-015",
        firstName: "Ohm",
        lastName: "Thitiwat",
        dob: "1997-05-05",
        gender: "Male",
        nric: "S9900112H",
        nationality: "Singapore",
        phone: "95567789",
        email: "ohm.thitiwat@csc.singaporehealth.sg",
        emergencyName: "Leo Thitiwat",
        emergencyPhone: "85554444",
        address: "122 Clementi Ave 5, #07-43, Singapore 120122",
        allergies: "None",
        medications: "None",
        conditions: "None",
        status: "Active"
    }
];

let allVisits = [
    { patientId: "PAT-001", purpose: "Surgery", date: "2026-09-23", summary: "Remove liver" },
    { patientId: "PAT-001", purpose: "Follow-up", date: "2026-08-12", summary: "Hypertension check" },
    { patientId: "PAT-001", purpose: "Consultation", date: "2026-05-07", summary: "Review condition" },
    { patientId: "PAT-001", purpose: "Health Checkup", date: "2026-04-15", summary: "Full screening" },

    { patientId: "PAT-002", purpose: "Consultation", date: "2026-07-10", summary: "Asthma review" },
    { patientId: "PAT-002", purpose: "Follow-up", date: "2026-03-02", summary: "Medication check" },

    { patientId: "PAT-003", purpose: "Consultation", date: "2026-06-19", summary: "Skin allergy check" },

    { patientId: "PAT-004", purpose: "Follow-up", date: "2026-10-01", summary: "Diabetes management" },
    { patientId: "PAT-004", purpose: "Lab Test", date: "2026-02-20", summary: "Glucose test" }
];

const pagination = new Pagination({
    data: [],
    rowsPerPage: 3,
    tbodyId: "patientTableBody",
    pageInfoId: "pageInfo",
    pageSelectId: "pageSelect",
    prevBtnId: "prevBtn",
    nextBtnId: "nextBtn",
    renderRow: renderPatientRow
});

function renderPatientRow(patient, index) {
    return `
    <tr class="hover:bg-slate-50">
        <td>${patient.id}</td>
        <td>${patient.firstName + " " + patient.lastName}</td>
        <td>${patient.gender}</td>
        <td>${formatDate(patient.dob)}</td>
        <td>${patient.phone}</td>
        <td><span class="status ${patient.status.toLowerCase()}">${patient.status}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <i class="view fa-solid fa-circle-info" onclick="viewPatient(${index})"></i>
                <span class="tooltip-text">View Details</span>
            </div>
            <div class="has-tooltip">
                <i class="edit fa-solid fa-pen-to-square" onclick="editPatient(${index})"></i>
                <span class="tooltip-text">Edit Details</span>
            </div>
        </td>
    </tr>`;
}

function viewPatient(index) {
    const p = pagination.paginatedData[index];

    document.getElementById("view-pid").innerText = p.id;
    document.getElementById("view-fullname").innerText = p.firstName + " " + p.lastName;
    document.getElementById("view-gender").innerText = p.gender;
    document.getElementById("view-dob").innerText = formatDate(p.dob);
    document.getElementById("view-nric").innerText = p.nric;
    document.getElementById("view-nationality").innerText = p.nationality;

    document.getElementById("view-phone").innerText = p.phone;
    document.getElementById("view-email").innerText = p.email;
    document.getElementById("view-address").innerText = p.address;
    document.getElementById("view-emergency").innerText = p.emergencyName + " | " + p.emergencyPhone;

    document.getElementById("view-allergies").innerText = p.allergies || "N.A.";
    document.getElementById("view-medications").innerText = p.medications || "N.A.";
    document.getElementById("view-conditions").innerText = p.conditions || "N.A.";

    const statusEl = document.getElementById("view-status");
    statusEl.innerText = p.status;
    statusEl.className = "status " + p.status.toLowerCase();

    // Visits Tab
    const { upcoming, past } = categorizeVisits(p.visits);

    const upEl = document.getElementById("view-upcoming-visits");
    upEl.innerHTML = upcoming.length ? upcoming.map(v => `
    <div class="card" onclick="window.location.href='/pages/Appointments.html'">
        <div class="top">
            <div class="purpose">${v.purpose}</div>
            <div class="date">${formatDate(v.date)}</div>
        </div>
        <div class="summary">${v.summary}</div>
        <div class="navigation"><i class="fa-solid fa-calendar-check"></i>Go to Appointment</div>
    </div>
    `).join("") : "<div class='description'>No upcoming visits</div>";

    const pastEl = document.getElementById("view-past-visits");
    pastEl.innerHTML = past.length ? past.map(v => `
    <div class="card" onclick="window.location.href='/pages/Medical-Records.html'">
        <div class="top">
            <div class="purpose">${v.purpose}</div>
            <div class="date">${formatDate(v.date)}</div>
        </div>
        <div class="summary">${v.summary}</div>
        <div class="navigation"><i class="fa-solid fa-file-medical"></i>View Record</div>
    </div>
    `).join("") : "<div class='description'>No past visits</div>";

    openModal("detailsModal");
}

function editPatient(index) {
    const p = pagination.paginatedData[index];

    const fullIndex = patients.findIndex(x => x.id === p.id);
    document.getElementById("edit-index").value = fullIndex;

    document.getElementById("edit-firstName").value = p.firstName;
    document.getElementById("edit-lastName").value = p.lastName;
    document.getElementById("edit-dob").value = p.dob;
    document.getElementById("edit-gender").value = p.gender.toLowerCase();
    document.getElementById("edit-nric").value = p.nric;
    document.getElementById("edit-nationality").value = p.nationality;
    document.getElementById("edit-phone").value = p.phone;
    document.getElementById("edit-email").value = p.email;
    document.getElementById("edit-emergencyName").value = p.emergencyName;
    document.getElementById("edit-emergencyPhone").value = p.emergencyPhone;
    document.getElementById("edit-address").value = p.address;
    document.getElementById("edit-allergies").value = p.allergies;
    document.getElementById("edit-medications").value = p.medications;
    document.getElementById("edit-conditions").value = p.conditions;

    openModal("editPatientModal");
}

function groupPatientVisits(patients, allVisits) {
    return patients.map(patient => {
        const patientVisits = allVisits.filter(
            visit => visit.patientId === patient.id
        );
        return {
            ...patient,
            visits: patientVisits || []
        };
    });
}

function categorizeVisits(visits) {
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    const upcoming = [];
    const past = [];

    visits.forEach(v => {
        const d = new Date(v.date);
        d.setHours(0, 0, 0, 0);
        d >= today ? upcoming.push(v) : past.push(v);
    });

    upcoming.sort((a, b) => new Date(a.date) - new Date(b.date));
    past.sort((a, b) => new Date(b.date) - new Date(a.date));

    return { upcoming, past };
}

function applySearch() {
    const keyword = document.getElementById("searchInput")?.value.toLowerCase() || "";
    return patients.filter(p =>
        p.id.toLowerCase().includes(keyword) ||
        p.firstName.toLowerCase().includes(keyword) ||
        p.lastName.toLowerCase().includes(keyword) ||
        p.phone.includes(keyword)
    );
}

function applyFilter(list) {
    const gender = document.getElementById("filter-gender")?.value || "";
    const status = document.getElementById("filter-status")?.value || "";

    return list.filter(item => {
        let match = true;
        if (gender) match = match && item.gender === gender;
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

function refreshPatientList() {
    let result = applySearch();
    result = applyFilter(result);
    result = applySort(result);

    const finalData = groupPatientVisits(result, allVisits);
    pagination.data = finalData;
    pagination.currentPage = 1;
    pagination.renderTable();
}

async function loadData() {
    const patientsWithVisits = groupPatientVisits(patients, allVisits);

    pagination.data = patientsWithVisits;
    pagination.renderTable();
}

document.addEventListener("DOMContentLoaded", () => {
    populateNationalityDropdown("add-nationality");
    populateNationalityDropdown("edit-nationality");

    // Calendar disable future date
    const today = new Date().toLocaleDateString('en-CA');
    const dateInputs = document.querySelectorAll('input[type="date"]');

    dateInputs.forEach(input => {
        input.setAttribute('max', today);
    });

    loadData();
});
