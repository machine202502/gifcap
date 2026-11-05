document.querySelectorAll("[data-lang-set]").forEach((link) => {
  link.addEventListener("click", () => {
    try { localStorage.setItem("gifcap-lang", link.dataset.langSet); } catch { /* keep the link */ }
  });
});

const shots = [...document.querySelectorAll(".shot img")];
if (shots.length) {
  const dialog = document.createElement("dialog");
  dialog.className = "lightbox";
  dialog.innerHTML = `<figure><img alt=""><figcaption></figcaption></figure>`;
  document.body.appendChild(dialog);
  const view = dialog.querySelector("img");
  const caption = dialog.querySelector("figcaption");
  shots.forEach((img) => {
    img.addEventListener("click", () => {
      view.src = img.currentSrc || img.src;
      view.alt = img.alt;
      caption.textContent = img.closest("figure")?.querySelector("figcaption")?.textContent || img.alt;
      dialog.showModal();
    });
  });
  dialog.addEventListener("click", () => dialog.close());
}

const TOKEN = "@@";
const nodes = [...document.querySelectorAll("[data-version], [data-hash-zip], [data-size-zip], [data-dl-zip]")];
if (nodes.some((node) => (node.textContent + (node.getAttribute("href") || "")).includes(TOKEN))) {
  document.querySelectorAll("[data-dl-zip]").forEach((a) => {
    a.href = "https://github.com/machine202502/gifcap/releases/latest";
  });
  fetch("https://api.github.com/repos/machine202502/gifcap/releases/latest")
    .then((response) => (response.ok ? response.json() : null))
    .then((release) => {
      const version = release ? (release.tag_name || "").replace(/^v/, "") : "";
      const zip = release && release.assets.find((asset) => asset.name.endsWith(".zip"));
      if (version) {
        document.querySelectorAll("[data-version]").forEach((node) => {
          node.textContent = node.textContent.split("@@VERSION@@").join(version);
        });
        if (zip) document.querySelectorAll("[data-dl-zip]").forEach((a) => { a.href = zip.browser_download_url; });
      }
      const size = document.querySelector("[data-size-zip]");
      const hash = document.querySelector("[data-hash-zip]");
      const unit = document.documentElement.dataset.sizeUnit || "MB";
      if (size && size.textContent.includes(TOKEN)) {
        size.textContent = zip ? `${(zip.size / 1048576).toFixed(1)} ${unit}` : "—";
      }
      if (hash && hash.textContent.includes(TOKEN)) hash.closest("p").style.display = "none";
    })
    .catch(() => {});
}
