// Five generated views. Only horizontal pointer position selects the head angle.
export function setupLogoMotion(slot, asset) {
  const logo = slot.querySelector('.brand-logo');
  const image = slot.querySelector('#brand-logo-image');
  if (!logo || !image) return;

  const names = ['left-far', 'left-near', 'center', 'right-near', 'right-far'];
  const frames = names.map(name => asset(`/assets/logo/moai-${name}.png`));
  const controller = new AbortController();
  const { signal } = controller;
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
  const canHover = window.matchMedia('(hover: hover)');
  let active = true;
  let ready = false;
  let current = 2;
  let pending = false;
  let pointerX;

  function show(index) {
    if (index === current) return;
    current = index;
    image.src = frames[index];
    logo.dataset.angle = names[index];
  }

  function reset() {
    pointerX = undefined;
    show(2);
  }

  // Decode every view before enabling interaction: switching never flashes blank.
  const preload = frames.map(src => {
    const frame = new Image();
    frame.src = src;
    return frame.decode();
  });
  Promise.all(preload).then(() => { ready = active; }).catch(() => {
    // Keep the static center view if any optional angle cannot be loaded.
    ready = false;
  });

  document.addEventListener('pointermove', event => {
    if (!ready || reducedMotion.matches || !canHover.matches || event.pointerType === 'touch') return;
    pointerX = event.clientX;
    if (pending) return;
    pending = true;
    requestAnimationFrame(() => {
      pending = false;
      if (!active || pointerX === undefined) return;
      const bounds = logo.getBoundingClientRect();
      if (!bounds.width) return;
      const horizontal = Math.max(0, Math.min(0.999999, (pointerX - bounds.left) / bounds.width));
      show(Math.floor(horizontal * frames.length));
    });
  }, { passive: true, signal });

  document.documentElement.addEventListener('pointerleave', reset, { signal });
  window.addEventListener('blur', reset, { signal });
  reducedMotion.addEventListener('change', reset, { signal });
  canHover.addEventListener('change', reset, { signal });
  return () => { controller.abort(); active = false; };
}
