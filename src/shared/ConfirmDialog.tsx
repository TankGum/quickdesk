import { useEffect, useRef } from "react";

import { t } from "./i18n";
import { Icon } from "./Icon";

/** Modal yes/no for destructive actions. Cancel has focus; Esc cancels. */
export function ConfirmDialog({
  title,
  message,
  confirmLabel,
  onConfirm,
  onCancel,
}: {
  title: string;
  message: string;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), []);
  return (
    <div className="dialog-backdrop confirm-backdrop" onMouseDown={(e) => e.target === e.currentTarget && onCancel()}>
      <div
        className="dialog confirm-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-label={title}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.preventDefault();
            e.stopPropagation();
            onCancel();
          }
        }}
      >
        <div className="confirm-body">
          <div className="confirm-icon">
            <Icon name="trash" size={20} />
          </div>
          <div>
            <h3>{title}</h3>
            <p className="muted">{message}</p>
          </div>
        </div>
        <div className="dialog-foot">
          <span className="spacer" />
          <button ref={cancel} className="btn" onClick={onCancel}>
            {t("common.cancel")}
          </button>
          <button className="btn danger-solid" onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
