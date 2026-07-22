const gallery = document.querySelector("[data-swing-gallery]");

if (gallery) {
  const dialog = document.querySelector("[data-swing-gallery-dialog]");
  const triggers = [...gallery.querySelectorAll("[data-swing-gallery-trigger]")];
  const close = dialog?.querySelector("[data-swing-gallery-close]");
  const previous = dialog?.querySelector("[data-swing-gallery-prev]");
  const next = dialog?.querySelector("[data-swing-gallery-next]");
  const image = dialog?.querySelector("[data-swing-gallery-image]");
  const video = dialog?.querySelector("[data-swing-gallery-video]");
  const position = dialog?.querySelector("[data-swing-gallery-position]");
  const caption = dialog?.querySelector("[data-swing-gallery-caption]");
  const initialSlug = gallery.dataset.swingGalleryInitial;
  let currentIndex = 0;
  let returnFocus = null;

  const indexForSlug = (slug) => triggers.findIndex((trigger) => trigger.dataset.swingGallerySlug === slug);
  const mediaUrl = (slug) => `/swing/${encodeURIComponent(slug)}`;
  const historyDepth = (state) => (Number.isInteger(state?.swingGalleryDepth) ? state.swingGalleryDepth : 0);
  const stateFor = (slug, depth) => ({
    ...(history.state ?? {}),
    swingGalleryMedia: slug,
    swingGalleryDepth: depth,
  });
  const slugFromState = (state) => state?.swingGalleryMedia;

  const clearVideo = () => {
    video.pause();
    video.removeAttribute("src");
    video.load();
  };

  const setMedia = (index) => {
    currentIndex = Math.max(0, Math.min(index, triggers.length - 1));
    const trigger = triggers[currentIndex];
    const { swingGalleryAlt: alt, swingGalleryCaption: text, swingGalleryKind: kind, swingGallerySrc: src } = trigger.dataset;
    const isVideo = kind === "Video";

    image.hidden = isVideo;
    video.hidden = !isVideo;

    if (isVideo) {
      image.removeAttribute("src");
      image.alt = "";
      video.src = src;
      video.setAttribute("aria-label", alt);
      video.load();
    } else {
      clearVideo();
      image.src = src;
      image.alt = alt;
    }

    position.textContent = `${kind} ${currentIndex + 1} of ${triggers.length}`;
    caption.textContent = text;
    previous.disabled = currentIndex === 0;
    next.disabled = currentIndex === triggers.length - 1;
  };

  const playVideo = () => {
    if (video.hidden) return;
    video.play().catch(() => {});
  };

  const openMedia = (index, { returnTo, updateHistory = false } = {}) => {
    const previousIndex = currentIndex;
    if (returnTo !== undefined) returnFocus = returnTo;
    setMedia(index);
    if (updateHistory && (!dialog.open || currentIndex !== previousIndex)) {
      const slug = triggers[currentIndex].dataset.swingGallerySlug;
      history.pushState(stateFor(slug, historyDepth(history.state) + 1), "", mediaUrl(slug));
    }
    if (!dialog.open) {
      dialog.showModal();
      close.focus();
    }
    document.documentElement.classList.add("swing-lightbox-open");
    document.body.classList.add("swing-lightbox-open");
    playVideo();
  };

  const closeFromUser = () => {
    const depth = historyDepth(history.state);
    if (slugFromState(history.state) && depth > 0) {
      history.go(-depth);
    } else {
      dialog.close();
    }
  };

  triggers.forEach((trigger, index) => {
    trigger.addEventListener("click", () => {
      openMedia(index, { returnTo: trigger, updateHistory: true });
    });
  });

  previous.addEventListener("click", () => openMedia(currentIndex - 1, { updateHistory: true }));
  next.addEventListener("click", () => openMedia(currentIndex + 1, { updateHistory: true }));
  close.addEventListener("click", closeFromUser);

  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) closeFromUser();
  });

  dialog.addEventListener("cancel", (event) => {
    event.preventDefault();
    closeFromUser();
  });

  dialog.addEventListener("keydown", (event) => {
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      openMedia(currentIndex - 1, { updateHistory: true });
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      openMedia(currentIndex + 1, { updateHistory: true });
    } else if (event.key === "Home") {
      event.preventDefault();
      openMedia(0, { updateHistory: true });
    } else if (event.key === "End") {
      event.preventDefault();
      openMedia(triggers.length - 1, { updateHistory: true });
    }
  });

  dialog.addEventListener("close", () => {
    clearVideo();
    image.removeAttribute("src");
    image.alt = "";
    document.documentElement.classList.remove("swing-lightbox-open");
    document.body.classList.remove("swing-lightbox-open");
    returnFocus?.focus();
    returnFocus = null;
  });

  window.addEventListener("popstate", (event) => {
    const index = indexForSlug(slugFromState(event.state));
    if (index >= 0) {
      openMedia(index);
    } else if (dialog.open) {
      dialog.close();
    }
  });

  const initialIndex = indexForSlug(initialSlug);
  if (initialIndex >= 0) {
    history.replaceState(stateFor(null, 0), "", "/swing");
    openMedia(initialIndex, { updateHistory: true });
  } else {
    history.replaceState(stateFor(null, 0), "", "/swing");
  }
}
