function openModal(modalId) {
    const modal = document.getElementById(modalId);
    modal.style.display = "flex";
}

function closeModal(button) {
    const modal = button.closest('.modal');
    modal.style.display = "none";

    const inputs = modal.querySelectorAll('input, textarea, select');
    inputs.forEach(input => {
        if (input.type === 'checkbox' || input.type === 'radio') {
            input.checked = false; 
        } else {
            input.value = '';   
        }
    });

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
    const parentDiv = button.closest("div");
    const inputs = parentDiv.querySelectorAll("input");
    const errorBox = parentDiv.querySelector('.error-box');
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

    let hasEmpty = false;
    let hasInvalidEmail = false;

    errorBox.style.display = 'none';

    inputs.forEach(input => {
        const value = input.value.trim();

        if (value === '') {
            hasEmpty = true;
        }

        if (input.type === 'email' && value !== '') {
            if (!emailRegex.test(value)) {
                hasInvalidEmail = true;
            }
        }
    });

    errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i>'
    if (hasEmpty) {
        errorBox.innerHTML += 'Please fill in all empty fields.';
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
