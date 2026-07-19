const home = document.querySelector("[data-felix-home]");
const hero = document.querySelector(".felix-hero");

if (home && hero) {
  const observer = new IntersectionObserver(
    ([entry]) => home.classList.toggle("is-visible", entry.intersectionRatio < 0.98),
    { threshold: [0.98] },
  );

  observer.observe(hero);
}
