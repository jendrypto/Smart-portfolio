import { useState } from 'react';

interface Props {
  isExposureTab: boolean;
}

function InfoFAB({ isExposureTab }: Props) {
  const [active, setActive] = useState(false);

  if (!isExposureTab) return null;

  const handleClick = () => {
    const next = !active;
    setActive(next);
    window.dispatchEvent(new CustomEvent('toggle-info-tooltips', { detail: { visible: next } }));
  };

  return (
    <button
      className={`info-fab ${active ? 'active' : ''}`}
      onClick={handleClick}
      title={active ? 'Hide metric explanations' : 'Show metric explanations'}
    >
      {active ? '\u2715' : '?'}
    </button>
  );
}

export default InfoFAB;
