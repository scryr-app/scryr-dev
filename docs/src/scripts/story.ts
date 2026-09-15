/** One scroll frame updates the camera and chapter art; no idle render loop. */
export function initStory() {
  const story = document.querySelector<HTMLElement>('.scryr-story');
  if (!story || story.classList.contains('is-ready')) return;
  const portal = story.querySelector<HTMLElement>('.map-portal');
  const copy = story.querySelector<HTMLElement>('.portal-copy');
  const mapCamera = story.querySelector<HTMLElement>('.map-camera');
  if (!portal || !copy) return;

  const skipLink = document.querySelector<HTMLAnchorElement>('a[href="#_top"]');
  if (skipLink) skipLink.href = '#product-title';

  const chapters = [...story.querySelectorAll<HTMLElement>('.story-chapter')];
  const chapterLinks = [...story.querySelectorAll<HTMLAnchorElement>('.chapter-nav a')];
  const preference = window.matchMedia('(prefers-reduced-motion: reduce)');
  const events = new AbortController();
  let frame = 0;
  let motion = !preference.matches;
  const clamp = (value: number) => Math.min(1, Math.max(0, value));
  const setProgress = (element: HTMLElement, name: string, value: number) => element.style.setProperty(name, value.toFixed(4));

  function update() {
    frame = 0;
    if (!story || !portal || !copy) return;
    const height = window.innerHeight;
    const rect = portal.getBoundingClientRect();
    const cinematic = height > 740 && window.innerWidth > 900;
    const camera = clamp(-rect.top / Math.max(1, rect.height - height));
    story.dataset.toolbar = rect.top < -80 ? 'visible' : 'hidden';
    story.dataset.chapters = rect.bottom < height * .6 ? 'visible' : 'hidden';
    if (motion) {
      setProgress(story, '--camera', camera);
      setProgress(story, '--intro', cinematic ? clamp(camera / .36) : 0);
      setProgress(story, '--discovery', cinematic ? clamp((camera - .5) / .3) : 0);
    }
    // Faded-out actions should never receive invisible keyboard focus.
    copy.inert = motion && cinematic && camera >= .36;
    let current = '';
    for (const chapter of chapters) {
      const box = chapter.getBoundingClientRect();
      if (box.top < height * .55) current = chapter.id;
      if (motion && box.bottom > 0 && box.top < height) {
        setProgress(chapter, '--progress', clamp((height - box.top) / (height + box.height)));
        setProgress(chapter, '--chart', clamp((height - box.top - box.height * .28) / (height * .55)));
      }
    }
    for (const link of chapterLinks) {
      if (link.hash === `#${current}`) link.setAttribute('aria-current', 'step');
      else link.removeAttribute('aria-current');
    }
  }
  function fitMap() {
    if (story && mapCamera) setProgress(story, '--map-fit', Math.min(1.2, mapCamera.clientWidth / 1150, mapCamera.clientHeight / 700));
    schedule();
  }
  function schedule() { if (!frame) frame = requestAnimationFrame(update); }
  function setMotion(enabled: boolean) {
    motion = enabled;
    if (!story) return;
    story.dataset.motion = enabled ? 'on' : 'off';
    schedule();
  }
  const observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        entry.target.classList.add('is-visible');
        observer.unobserve(entry.target);
      }
    }
  }, { threshold: .08 });
  story.querySelectorAll<HTMLElement>('[data-reveal]').forEach((element, i) => {
    element.style.setProperty('--stagger', `${(i % 3) * 90}ms`);
    observer.observe(element);
  });
  story.classList.add('is-ready');
  setMotion(motion);
  fitMap();
  update();
  preference.addEventListener('change', event => setMotion(!event.matches), { signal: events.signal });
  window.addEventListener('scroll', schedule, { passive: true, signal: events.signal });
  window.addEventListener('resize', fitMap, { signal: events.signal });
  document.addEventListener('astro:before-swap', () => {
    events.abort();
    observer.disconnect();
    cancelAnimationFrame(frame);
  }, { once: true });
}
