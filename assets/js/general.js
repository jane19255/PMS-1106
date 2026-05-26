document.addEventListener("DOMContentLoaded", function () {
  // 1. Highlight the current page in the sidebar menu
  const pageTitle = document.title.split(" - ")[0].trim();
  const navLinks = document.querySelectorAll(".mega-menu a");

  navLinks.forEach((link) => {
    const linkText = link.textContent.trim();
    if (linkText === pageTitle) {
      link.classList.add("selected");
    }
  });

  // 2. Calendar disable future date
  const today = new Date().toLocaleDateString("en-CA");
  const dateInputs = document.querySelectorAll('input[type="date"]');

  dateInputs.forEach((input) => {
    input.setAttribute("max", today);
  });
});

function burgerMenu() {
  const menu = document.querySelector(".mega-menu");
  const btn = document.getElementById("menuBtn");
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
  const modal = button.closest(".modal");
  modal.style.display = "none";
  document.documentElement.classList.remove("no-scroll");
  return true;
}

function showToast(msg) {
  const toast = document.getElementById("toast");
  toast.innerHTML = "<i class='fa-solid fa-check-circle'></i>" + msg;

  toast.classList.add("show");
  setTimeout(() => {
    toast.classList.remove("show");
  }, 3000);
  return true;
}

function verifyInput(button) {
  const parentDiv = button.closest("div.card");
  const errorBox = parentDiv.querySelector(".error-box");

  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  const sgPhoneRegex = /^[89]\d{7}$/; // Starts with 8/9, 8 number

  let hasEmpty = false;
  let hasInvalidEmail = false;
  let hasInvalidPhone = false;

  errorBox.style.display = "none";
  errorBox.innerHTML = "";

  const requiredInputs = parentDiv.querySelectorAll(
    "input[required], textarea[required], select[required]",
  );

  requiredInputs.forEach((input) => {
    let value = input.value.trim();
    if (input.type === "tel") {
      value = value.replace(/\s+/g, "");
    }

    if (value === "") {
      hasEmpty = true;
    }

    if (input.type === "email" && value !== "") {
      if (!emailRegex.test(value)) {
        hasInvalidEmail = true;
      }
    }

    if (input.type === "tel" && value !== "") {
      if (!sgPhoneRegex.test(value)) {
        hasInvalidPhone = true;
      }
    }
  });

  errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i> ';

  if (hasEmpty) {
    errorBox.innerHTML += "Please fill in all empty fields.";
    errorBox.style.display = "flex";
    return false;
  } else if (hasInvalidPhone) {
    errorBox.innerHTML += "Invalid phone number. Please try again.";
    errorBox.style.display = "flex";
    return false;
  } else if (hasInvalidEmail) {
    errorBox.innerHTML += "Invalid email address. Please try again.";
    errorBox.style.display = "flex";
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
  const modal = button.closest(".modal");
  const inputs = modal.querySelectorAll("input, textarea, select");

  inputs.forEach((input) => {
    if (input.type === "checkbox" || input.type === "radio") {
      input.checked = false;
    } else if (input.tagName === "SELECT") {
      input.selectedIndex = 0;
    } else {
      input.value = "";
    }
  });
}
