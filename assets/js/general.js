// Header and Footer
document.addEventListener('DOMContentLoaded', function () {

    fetch('header.html')
        .then(r => r.text())
        .then(html => {
            const h = document.getElementById('header-placeholder');
            if (h) h.innerHTML = html;

            const pageTitle = document.title.split(" - ")[0].trim();
            const navLinks = document.querySelectorAll(".mega-menu a");

            navLinks.forEach(link => {
                const linkText = link.textContent.trim();
                if (linkText === pageTitle) {
                    link.classList.add("selected");
                }
            });
        });

    fetch('footer.html')
        .then(r => r.text())
        .then(html => {
            const f = document.getElementById('footer-placeholder');
            if (f) f.innerHTML = html;
        });
});

function burgerMenu() {
    const menu = document.querySelector(".mega-menu");
    const btn = document.getElementById("menuBtn")
    const icon = btn.classList.contains("fa-bars");

    menu.classList.toggle("show");
    btn.classList.toggle("fa-xmark", icon);
    btn.classList.toggle("fa-bars", !icon);
}

function openModal(modalId) {
    const modal = document.getElementById(modalId);
    modal.style.display = "flex";
    document.documentElement.classList.add("no-scroll");
}

function closeModal(button) {
    const modal = button.closest('.modal');
    modal.style.display = "none";
    document.documentElement.classList.remove("no-scroll");
    return true;
}

function showToast(msg) {
    const toast = document.getElementById('toast');
    toast.innerHTML = "<i class='fa-solid fa-check-circle'></i>" + msg;

    toast.classList.add("show");
    setTimeout(() => { toast.classList.remove("show"); }, 3000);
    return true;
}

function verifyInput(button) {
    const parentDiv = button.closest("div.card");
    const errorBox = parentDiv.querySelector('.error-box');

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    const sgPhoneRegex = /^[89]\d{7}$/; // Starts with 8/9, 8 mumber

    let hasEmpty = false;
    let hasInvalidEmail = false;
    let hasInvalidPhone = false;

    errorBox.style.display = 'none';
    errorBox.innerHTML = '';

    const requiredInputs = parentDiv.querySelectorAll('input[required], textarea[required], select[required]');

    requiredInputs.forEach(input => {
        let value = input.value.trim();
        if (input.type === 'tel') {
            value = value.replace(/\s+/g, '');
        }

        if (value === '') {
            hasEmpty = true;
        }

        if (input.type === 'email' && value !== '') {
            if (!emailRegex.test(value)) {
                hasInvalidEmail = true;
            }
        }

        if (input.type === 'tel' && value !== '') {
            if (!sgPhoneRegex.test(value)) {
                hasInvalidPhone = true;
            }
        }
    });

    errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i> ';

    if (hasEmpty) {
        errorBox.innerHTML += 'Please fill in all empty fields.';
        errorBox.style.display = 'flex';
        return false;
    } else if (hasInvalidPhone) {
        errorBox.innerHTML += 'Invalid phone number. Please try again.';
        errorBox.style.display = 'flex';
        return false;
    } else if (hasInvalidEmail) {
        errorBox.innerHTML += 'Invalid email address. Please try again.';
        errorBox.style.display = 'flex';
        return false;
    }

    return true;
}

function showPassword(button) {
    const input = button.closest(".field").querySelector("input");
    const isPassword = input.type === "password";

    input.type = isPassword ? "text" : "password";

    button.classList.toggle("fa-eye", isPassword);
    button.classList.toggle("fa-eye-slash", !isPassword);
}

function clearInput(button) {
    const modal = button.closest('.modal');
    const inputs = modal.querySelectorAll('input, textarea, select');

    inputs.forEach(input => {
        if (input.type === 'checkbox' || input.type === 'radio') {
            input.checked = false;
        } else if (input.tagName === 'SELECT') {
            input.selectedIndex = 0;
        } else {
            input.value = '';
        }
    });
}

function openTab(button, target) {
    const targetTab = document.getElementById(target);
    const selectedButton = document.querySelector(".tab-section .tab.selected");
    const tabs = document.querySelectorAll(".tab-content");

    selectedButton.classList.remove("selected");
    tabs.forEach(tab => {
        tab.classList.remove("show");
    });

    button.classList.add("selected");
    targetTab.classList.add("show");
}

function formatDate(dateString) {
    if (!dateString) return 'N/A';

    return new Date(dateString).toLocaleDateString('en-GB', {
        day: 'numeric',
        month: 'long',
        year: 'numeric'
    });
}

class Pagination {
    constructor(config) {
        this.data = config.data || [];
        this.rowsPerPage = parseInt(config.rowsPerPage || 5);
        this.currentPage = 1;
        this.renderRow = config.renderRow;
        this.tbodyId = config.tbodyId;
        this.pageInfoId = config.pageInfoId;
        this.pageSelectId = config.pageSelectId;
        this.prevBtnId = config.prevBtnId;
        this.nextBtnId = config.nextBtnId;
    }

    get totalPages() {
        return Math.ceil(this.data.length / this.rowsPerPage);
    }

    get paginatedData() {
        const start = (this.currentPage - 1) * this.rowsPerPage;
        const end = start + this.rowsPerPage;
        return this.data.slice(start, end);
    }

    renderTable() {
        const tbody = document.getElementById(this.tbodyId);
        tbody.innerHTML = '';
        this.paginatedData.forEach((item, idx) => {
            tbody.innerHTML += this.renderRow(item, idx);
        });
        this.updateAll();
    }

    updateAll() {
        this.updatePaginationUI();
        this.toggleButtons();
        this.updateInfo();
    }

    updatePaginationUI() {
        const select = document.getElementById(this.pageSelectId);
        select.innerHTML = '';
        for (let i = 1; i <= this.totalPages; i++) {
            const opt = document.createElement("option");
            opt.value = i;
            opt.textContent = `${i} of ${this.totalPages}`;
            select.appendChild(opt);
        }
        select.value = this.currentPage;
    }

    toggleButtons() {
        const prevBtn = document.getElementById(this.prevBtnId);
        const nextBtn = document.getElementById(this.nextBtnId);

        prevBtn.disabled = this.currentPage === 1;
        nextBtn.disabled = this.currentPage === this.totalPages;
    }

    updateInfo() {
        const start = (this.currentPage - 1) * this.rowsPerPage + 1;
        const end = Math.min(this.currentPage * this.rowsPerPage, this.data.length);
        document.getElementById(this.pageInfoId).innerText = `Showing ${start} - ${end} of ${this.data.length}`;
    }

    goToPage(page) {
        this.currentPage = parseInt(page);
        this.renderTable();
    }

    prev() {
        if (this.currentPage > 1) {
            this.currentPage--;
            this.renderTable();
        }
        this.toggleButtons();
    }

    next() {
        if (this.currentPage < this.totalPages) {
            this.currentPage++;
            this.renderTable();
        }
        this.toggleButtons();
    }

    changeRowsPerPage(value) {
        this.rowsPerPage = parseInt(value);
        this.currentPage = 1;
        this.renderTable();
    }
}