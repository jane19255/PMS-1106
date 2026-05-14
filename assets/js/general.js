function openModal(modalId) {
    const modal = document.getElementById(modalId);
    modal.style.display = "flex";
}

function closeModal(button) {
    const modal = button.closest('.modal');
    modal.style.display = "none";
}

function showToast(msg) {
    const toast = document.getElementById('toast');
    toast.innerHTML = "<i class='fa-solid fa-check-circle'></i>" + msg;

    toast.classList.add("show");
    setTimeout(() => {toast.classList.remove("show");}, 3000);
}
