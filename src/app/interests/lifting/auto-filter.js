// Progressive enhancement for the native GET form. The server still owns
// query validation and rendering; this only schedules the next navigation.

const form = document.querySelector("form[data-lifting-filters]");

if (form instanceof HTMLFormElement) {
  let pendingNavigation;

  const encode = (value) =>
    encodeURIComponent(value).replace(/[!'()*]/g, (character) =>
      `%${character.charCodeAt(0).toString(16).toUpperCase()}`,
    );

  const targetUrl = () => {
    const pairs = [];
    for (const [key, rawValue] of new FormData(form)) {
      const value = String(rawValue).trim();
      if (value === "") continue;
      if (key === "page" && value === "1") continue;
      if (key === "per_page" && value === "10") continue;
      pairs.push(`${encode(key)}=${encode(value)}`);
    }
    const query = pairs.length === 0 ? "" : `?${pairs.join("&")}`;
    return `/lifting/log${query}#set-log`;
  };

  const navigate = () => {
    pendingNavigation = undefined;
    const target = targetUrl();
    const current = `${location.pathname}${location.search}${location.hash}`;
    if (target !== current) location.assign(target);
  };

  const schedule = (delay) => {
    window.clearTimeout(pendingNavigation);
    pendingNavigation = window.setTimeout(navigate, delay);
  };

  form.addEventListener("input", (event) => {
    const target = event.target;
    const immediate =
      target instanceof HTMLSelectElement ||
      (target instanceof HTMLInputElement &&
        (target.type === "checkbox" || target.type === "date"));
    schedule(immediate ? 0 : 260);
  });

  // `input` covers selects in current browsers; `change` is a cheap fallback.
  form.addEventListener("change", (event) => {
    if (event.target instanceof HTMLSelectElement) schedule(0);
  });

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    window.clearTimeout(pendingNavigation);
    navigate();
  });
}
