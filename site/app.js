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
const nodes = [...document.querySelectorAll("[data-version], [data-hash-zip], [data-size-zip], [data-hash-setup], [data-size-setup], [data-dl-zip], [data-dl-setup]")];
if (nodes.some((node) => (node.textContent + (node.getAttribute("href") || "")).includes(TOKEN))) {
  const latest = "https://github.com/machine202502/gifcap/releases/latest";
  document.querySelectorAll("[data-dl-zip], [data-dl-setup]").forEach((a) => { a.href = latest; });
  fetch("https://api.github.com/repos/machine202502/gifcap/releases/latest")
    .then((response) => (response.ok ? response.json() : null))
    .then((release) => {
      const version = release ? (release.tag_name || "").replace(/^v/, "") : "";
      const zip = release && release.assets.find((asset) => asset.name.endsWith(".zip"));
      const setup = release && release.assets.find((asset) => asset.name.endsWith("-setup.exe"));
      if (version) {
        document.querySelectorAll("[data-version]").forEach((node) => {
          node.textContent = node.textContent.split("@@VERSION@@").join(version);
        });
      }
      const unit = document.documentElement.dataset.sizeUnit || "MB";
      const bind = (asset, link, size, hash) => {
        if (asset) document.querySelectorAll(link).forEach((a) => { a.href = asset.browser_download_url; });
        const sizeNode = document.querySelector(size);
        const hashNode = document.querySelector(hash);
        if (sizeNode && sizeNode.textContent.includes(TOKEN)) {
          sizeNode.textContent = asset ? `${(asset.size / 1048576).toFixed(1)} ${unit}` : "—";
        }
        if (hashNode && hashNode.textContent.includes(TOKEN)) hashNode.closest("p").style.display = "none";
      };
      bind(zip, "[data-dl-zip]", "[data-size-zip]", "[data-hash-zip]");
      bind(setup, "[data-dl-setup]", "[data-size-setup]", "[data-hash-setup]");
    })
    .catch(() => {});
}
