// Keep the same interactive logo visible in the desktop gutter after scrolling.
export function setupLogoDock(slot) {
  const brand = slot?.querySelector('.brand');
  if (!slot || !brand) return;
  const wideScreen = window.matchMedia('(min-width: 1100px)');
  const controller = new AbortController();
  const { signal } = controller;
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
  let movement;
  const footer = document.querySelector('.site-footer');
  const footerName = footer?.querySelector('.footer-name');

  function update() {
    const passedLogo = slot.getBoundingClientRect().bottom <= 0;
    const docked = wideScreen.matches && passedLogo;
    const atFooter = docked && footer && footer.getBoundingClientRect().top < window.innerHeight;
    const before = brand.getBoundingClientRect();
    if (atFooter && footerName) {
      const anchor = footerName.getBoundingClientRect();
      brand.style.setProperty('--footer-logo-left', `${anchor.left}px`);
      brand.style.setProperty('--footer-logo-bottom', `${window.innerHeight - anchor.bottom}px`);
    }
    if (brand.classList.contains('is-docked') === docked &&
        brand.classList.contains('is-footer-docked') === Boolean(atFooter)) return;

    // FLIP keeps the same mouse-tracking logo and smoothly morphs its position
    // and size between the original header and the corner, without layout shifts.
    movement?.cancel();
    brand.classList.toggle('is-docked', docked);
    brand.classList.toggle('is-footer-docked', Boolean(atFooter));
    const after = brand.getBoundingClientRect();
    if (reducedMotion.matches || !brand.animate || !after.width || !after.height) return;
    movement = brand.animate([
      { transform: `translate(${before.left - after.left}px, ${before.top - after.top}px) scale(${before.width / after.width}, ${before.height / after.height})` },
      { transform: 'translate(0, 0) scale(1, 1)' }
    ], {
      duration: 1200,
      easing: 'cubic-bezier(0.22, 1, 0.36, 1)',
    });
  }

  function measure() {
    // Preserve the original space so docking never shifts content or scroll.
    movement?.cancel();
    brand.classList.remove('is-docked', 'is-footer-docked');
    slot.style.height = `${brand.getBoundingClientRect().height}px`;
    update();
  }

  const observer = new IntersectionObserver(update);
  observer.observe(slot);
  if (footer) observer.observe(footer);
  window.addEventListener('scroll', update, { passive: true, signal });
  window.addEventListener('resize', measure, { passive: true, signal });
  wideScreen.addEventListener('change', measure, { signal });
  reducedMotion.addEventListener('change', () => movement?.cancel(), { signal });
  measure();
  return () => { controller.abort(); observer.disconnect(); movement?.cancel(); };
}
