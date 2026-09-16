/** One scroll frame updates the camera and chapter art; no idle render loop. */
export function initStory() {
  const story = document.querySelector<HTMLElement>('.scryr-story');
  if (!story || story.classList.contains('is-ready')) return;
  const portal = story.querySelector<HTMLElement>('.map-portal');
  const copy = story.querySelector<HTMLElement>('.portal-copy');
  const mapCamera = story.querySelector<HTMLElement>('.map-camera');
  const toolbar = document.querySelector<HTMLElement>('header.header');
  if (!portal || !copy) return;

  const skipLink = document.querySelector<HTMLAnchorElement>('a[href="#_top"]');
  if (skipLink) skipLink.href = '#product-title';

  const chapters = [...story.querySelectorAll<HTMLElement>('.story-chapter')];
  const rectanglePunchline = story.querySelector<HTMLElement>('.rectangle-punchline');
  const rectangles = [...story.querySelectorAll<HTMLElement>('.pencil-caption')];
  const annotationStage = story.querySelector<HTMLElement>('.retro-desktop');
  const annotationTargets = [...story.querySelectorAll<SVGRectElement>('[data-rectangle-target]')];
  const annotationArrows = [...story.querySelectorAll<SVGPathElement>('.annotation-arrow')];
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
    // Reveal the toolbar gradually as the hero leaves the viewport.
    const toolbarProgress = clamp((height * .9 - rect.bottom) / (height * .6));
    const toolbarReveal = toolbar?.querySelector(':popover-open') ? 1 : motion ? toolbarProgress : Number(toolbarProgress > 0);
    story.dataset.toolbar = toolbarReveal > 0 ? 'visible' : 'hidden';
    if (toolbar) {
      setProgress(toolbar, '--toolbar-reveal', toolbarReveal);
      toolbar.inert = toolbarReveal < .05;
    }
    if (motion) {
      setProgress(story, '--camera', camera);
      setProgress(story, '--intro', cinematic ? clamp(camera / .36) : 0);
      setProgress(story, '--discovery', cinematic ? clamp((camera - .5) / .3) : 0);
    }
    // Faded-out actions should never receive invisible keyboard focus.
    copy.inert = motion && cinematic && camera >= .36;
    for (const chapter of chapters) {
      const box = chapter.getBoundingClientRect();
      if (motion && box.bottom > 0 && box.top < height) {
        setProgress(chapter, '--progress', clamp((height - box.top) / (height + box.height)));
        setProgress(chapter, '--chart', clamp((height - box.top - box.height * .28) / (height * .55)));
      }
    }
    if (rectanglePunchline && annotationStage) {
      const stage = annotationStage.getBoundingClientRect();
      const top = rectanglePunchline.getBoundingClientRect().top;
      const celebration = motion ? clamp(((height * .95 - stage.top) / (height * .72) - .9) / .4) : 1;
      setProgress(annotationStage, '--celebration', celebration);
      setProgress(annotationStage, '--yay-writing', clamp(celebration * 2));
      rectangles.forEach((caption, index) => {
        const progress = motion ? clamp((height * .95 - top) / (height * .72) - index * .22) : 1;
        const arrival = clamp(progress / .55);
        setProgress(caption, '--arrival', 1 - Math.pow(1 - arrival, 3));
        const writing = clamp(progress / .7);
        setProgress(caption, '--writing', writing);
        setProgress(caption, '--pencil-visible', progress > 0 && writing < 1 ? 1 : 0);
        const target = annotationTargets[index];
        const arrow = annotationArrows[index];
        if (!target || !arrow) return;
        const label = caption.getBoundingClientRect();
        const node = target.getBoundingClientRect();
        const x = label.left + label.width / 2 - stage.left;
        const y = label.bottom + 5 - stage.top;
        const endX = node.left + node.width / 2 - stage.left;
        const endY = node.top - stage.top;
        const bendX = index === 0 ? Math.min(x, endX) - 35 : Math.max(x, endX) + 35;
        arrow.setAttribute('d', `M${x} ${y} C${bendX} ${y}, ${bendX} ${endY - 35}, ${endX} ${endY}`);
        const draw = clamp((progress - .7) / .3);
        arrow.style.setProperty('--arrow-progress', draw.toFixed(4));
        arrow.style.setProperty('--arrow-visible', draw > 0 ? '1' : '0');
      });
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
