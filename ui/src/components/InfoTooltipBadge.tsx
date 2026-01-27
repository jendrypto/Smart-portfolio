import { useState, useEffect, useRef } from 'react';

interface Props {
  text: string;
}

function InfoTooltipBadge({ text }: Props) {
  const [visible, setVisible] = useState(false);
  const [showPopup, setShowPopup] = useState(false);
  const badgeRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      setVisible(detail.visible);
      if (!detail.visible) setShowPopup(false);
    };
    window.addEventListener('toggle-info-tooltips', handler);
    return () => window.removeEventListener('toggle-info-tooltips', handler);
  }, []);

  if (!visible) return null;

  return (
    <span className="info-tooltip-badge-wrapper" ref={badgeRef}>
      <span
        className="info-tooltip-badge"
        onClick={(e) => {
          e.stopPropagation();
          setShowPopup(!showPopup);
        }}
      >
        i
      </span>
      {showPopup && (
        <div className="info-tooltip-popup" onClick={(e) => e.stopPropagation()}>
          {text}
        </div>
      )}
    </span>
  );
}

export default InfoTooltipBadge;
