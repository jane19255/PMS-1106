document.addEventListener("DOMContentLoaded", () => {
  const selectElement = document.getElementById("nationalityDropdown");
  if (!selectElement) return;

  const nationalities = [
    "Afghan", "Albanian", "Algerian", "American", "Andorran", "Angolan", "Argentine", "Armenian",
    "Australian", "Austrian", "Azerbaijani", "Bahamian", "Bahraini", "Bangladeshi", "Barbadian",
    "Belarusian", "Belgian", "Belizian", "Beninese", "Bhutanese", "Bolivian", "Bosnian", "Brazilian",
    "British", "Bruneian", "Bulgarian", "Burmese", "Cambodian", "Cameroonian", "Canadian", "Chilean",
    "Chinese", "Colombian", "Congolese", "Costa Rican", "Croatian", "Cuban", "Cypriot", "Czech",
    "Danish", "Dominican", "Dutch", "Ecuadorian", "Egyptian", "Emirati", "Estonian", "Ethiopian",
    "Fijian", "Filipino", "Finnish", "French", "Georgian", "German", "Ghanaian", "Greek", "Guatemalan",
    "Haitian", "Honduran", "Hungarian", "Icelandic", "Indian", "Indonesian", "Iranian", "Iraqi",
    "Irish", "Israeli", "Italian", "Ivorian", "Jamaican", "Japanese", "Jordanian", "Kazakh", "Kenyan",
    "Korean", "Kuwaiti", "Laotian", "Latvian", "Lebanese", "Liberian", "Libyan", "Lithuanian",
    "Luxembourger", "Macedonian", "Malagasy", "Malawian", "Maldivian", "Malian", "Maltese", "Mauritian",
    "Mexican", "Moldovan", "Monacan", "Mongolian", "Moroccan", "Mozambican", "Nepalese", "New Zealander",
    "Nicaraguan", "Nigerian", "Norwegian", "Omani", "Pakistani", "Panamanian", "Paraguayan", "Peruvian",
    "Polish", "Portuguese", "Qatari", "Romanian", "Russian", "Saudi", "Scottish", "Senegalese",
    "Serbian", "Sierra Leonean", "Slovak", "Slovenian", "Somali", "South African", "Spanish",
    "Sri Lankan", "Sudanese", "Swedish", "Swiss", "Syrian", "Taiwanese", "Tajik", "Tanzanian",
    "Thai", "Togolese", "Tunisian", "Turkish", "Ugandan", "Ukrainian", "Uruguayan", "Uzbek",
    "Venezuelan", "Vietnamese", "Welsh", "Yemeni", "Zambian", "Zimbabwean"
  ];

  nationalities.forEach(nation => {
    const option = document.createElement("option");
    option.value = nation;
    option.textContent = nation;
    selectElement.appendChild(option);
  });
});

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