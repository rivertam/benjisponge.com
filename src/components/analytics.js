(function () {
  "use strict";

  var dnt = String(navigator.doNotTrack || window.doNotTrack || "").toLowerCase();
  if (
    navigator.webdriver ||
    navigator.globalPrivacyControl === true ||
    dnt === "1" ||
    dnt === "yes" ||
    !window.crypto ||
    typeof window.crypto.randomUUID !== "function" ||
    typeof window.fetch !== "function"
  ) {
    return;
  }

  var endpoint = "/api/analytics/events";
  var bootstrapKey = "benjisponge.analytics.bootstrap.v2";
  var bootstrapId = window.crypto.randomUUID();
  try {
    var storedBootstrap =
      window.sessionStorage.getItem(bootstrapKey) ||
      window.sessionStorage.getItem("benjisponge.analytics.session.v1");
    if (
      typeof storedBootstrap === "string" &&
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(
        storedBootstrap
      )
    ) {
      bootstrapId = storedBootstrap.toLowerCase();
    }
    window.sessionStorage.setItem(bootstrapKey, bootstrapId);
  } catch (_error) {
    // A document-scoped nonce still works when storage is unavailable.
  }
  var identityBootstrap = document.getElementById(
    "analytics-private-bootstrap"
  );
  if (identityBootstrap) {
    identityBootstrap.value = bootstrapId;
  }
  var engagementId = window.crypto.randomUUID();

  function eventPayload(kind, id) {
    return {
      version: 2,
      id: id || window.crypto.randomUUID(),
      bootstrap_id: bootstrapId,
      kind: kind
    };
  }

  function beacon(body) {
    if (typeof navigator.sendBeacon !== "function") {
      return false;
    }
    try {
      return navigator.sendBeacon(
        endpoint,
        new Blob([body], { type: "application/json" })
      );
    } catch (_error) {
      return false;
    }
  }

  function send(payload, urgent) {
    var body = JSON.stringify(payload);
    if (urgent && beacon(body)) {
      return null;
    }
    return window.fetch(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: body,
      credentials: "same-origin",
      keepalive: true
    }).catch(function () {
      beacon(body);
      return null;
    });
  }

  function forgetBootstrap(response) {
    if (!response || !response.ok) {
      return;
    }
    try {
      window.sessionStorage.removeItem(bootstrapKey);
      window.sessionStorage.removeItem("benjisponge.analytics.session.v1");
      window.sessionStorage.removeItem("benjisponge.analytics.session.v1.seen");
    } catch (_error) {
      // Storage was only a bridge until the hardened cookie response arrived.
    }
  }

  function acquisitionReferrer() {
    if (!document.referrer) {
      return null;
    }
    try {
      var referrer = new URL(document.referrer);
      if (referrer.protocol !== "http:" && referrer.protocol !== "https:") {
        return null;
      }
      return referrer.origin === window.location.origin
        ? referrer.origin + referrer.pathname
        : referrer.origin;
    } catch (_error) {
      return null;
    }
  }

  function sendPageview() {
    var payload = eventPayload("pageview");
    var referrer = acquisitionReferrer();
    var width = Math.round(window.innerWidth || 0);
    if (referrer) {
      payload.referrer = referrer;
    }
    try {
      payload.timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
    } catch (_error) {
      // Timezone is an optional coarse dimension.
    }
    if (width > 0) {
      payload.viewport_width = width;
    }
    return send(payload, false);
  }

  var visibleAt =
    document.visibilityState === "visible" ? performance.now() : null;
  var visibleMilliseconds = 0;
  var maxScrollPercent = 0;
  var scrollQueued = false;
  var lcpMilliseconds = null;
  var cls = 0;
  var clsWindow = 0;
  var clsWindowStart = 0;
  var clsWindowLast = 0;
  var lcpObserver = null;
  var clsObserver = null;

  function pauseVisible() {
    if (visibleAt !== null) {
      visibleMilliseconds += Math.max(0, performance.now() - visibleAt);
      visibleAt = null;
    }
  }

  function resumeVisible() {
    if (visibleAt === null && document.visibilityState === "visible") {
      visibleAt = performance.now();
    }
  }

  function measureScroll() {
    var root = document.documentElement;
    var body = document.body;
    var viewport = window.innerHeight || root.clientHeight || 0;
    var top = window.scrollY || root.scrollTop || body.scrollTop || 0;
    var height = Math.max(
      root.scrollHeight,
      root.offsetHeight,
      body.scrollHeight,
      body.offsetHeight,
      viewport
    );
    var percent = height > 0 ? ((top + viewport) / height) * 100 : 0;
    maxScrollPercent = Math.max(
      maxScrollPercent,
      Math.max(0, Math.min(100, percent))
    );
  }

  function scheduleScroll() {
    if (scrollQueued) {
      return;
    }
    scrollQueued = true;
    window.requestAnimationFrame(function () {
      scrollQueued = false;
      measureScroll();
    });
  }

  function collectLcp(entries) {
    entries.forEach(function (entry) {
      lcpMilliseconds = entry.startTime;
    });
  }

  function collectCls(entries) {
    entries.forEach(function (entry) {
      if (entry.hadRecentInput) {
        return;
      }
      if (
        clsWindowStart === 0 ||
        entry.startTime - clsWindowLast > 1000 ||
        entry.startTime - clsWindowStart > 5000
      ) {
        clsWindowStart = entry.startTime;
        clsWindow = entry.value;
      } else {
        clsWindow += entry.value;
      }
      clsWindowLast = entry.startTime;
      cls = Math.max(cls, clsWindow);
    });
  }

  function observePerformance() {
    if (typeof window.PerformanceObserver !== "function") {
      return;
    }
    try {
      lcpObserver = new PerformanceObserver(function (list) {
        collectLcp(list.getEntries());
      });
      lcpObserver.observe({ type: "largest-contentful-paint", buffered: true });
    } catch (_error) {
      lcpObserver = null;
    }
    try {
      clsObserver = new PerformanceObserver(function (list) {
        collectCls(list.getEntries());
      });
      clsObserver.observe({ type: "layout-shift", buffered: true });
    } catch (_error) {
      clsObserver = null;
    }
  }

  function navigationMilliseconds() {
    var entry = performance.getEntriesByType("navigation")[0];
    return entry && Number.isFinite(entry.duration) ? entry.duration : null;
  }

  function flushEngagement() {
    pauseVisible();
    measureScroll();
    if (lcpObserver) {
      collectLcp(lcpObserver.takeRecords());
    }
    if (clsObserver) {
      collectCls(clsObserver.takeRecords());
    }

    var payload = eventPayload("engagement", engagementId);
    var navigation = navigationMilliseconds();
    payload.engagement_ms = Math.min(
      7200000,
      Math.max(0, Math.round(visibleMilliseconds))
    );
    payload.scroll_percent = Math.round(maxScrollPercent);
    if (Number.isFinite(lcpMilliseconds)) {
      payload.lcp_ms = Math.round(lcpMilliseconds);
    }
    payload.cls_milli = Math.max(0, Math.round(cls * 1000));
    if (Number.isFinite(navigation)) {
      payload.navigation_ms = Math.round(navigation);
    }
    send(payload, true);
  }

  function clicked(event) {
    if (typeof event.button === "number" && event.button > 0) {
      return;
    }
    var target =
      event.target instanceof Element ? event.target.closest("a[href]") : null;
    if (!target) {
      return;
    }
    try {
      var destination = new URL(target.href, document.baseURI);
      if (
        (destination.protocol === "http:" ||
          destination.protocol === "https:") &&
        destination.origin !== window.location.origin
      ) {
        var payload = eventPayload("outbound");
        payload.target = destination.origin;
        send(payload, false);
      }
    } catch (_error) {
      // Malformed and non-HTTP destinations are not analytics events.
    }
  }

  observePerformance();
  measureScroll();
  var pageviewRequest = sendPageview();
  if (pageviewRequest) {
    pageviewRequest.then(forgetBootstrap);
  }

  window.addEventListener("scroll", scheduleScroll, { passive: true });
  window.addEventListener("resize", scheduleScroll);
  document.addEventListener("click", clicked);
  document.addEventListener("visibilitychange", function () {
    if (document.visibilityState === "hidden") {
      flushEngagement();
    } else {
      resumeVisible();
    }
  });
  window.addEventListener("pagehide", flushEngagement);
  window.addEventListener("pageshow", resumeVisible);
})();
