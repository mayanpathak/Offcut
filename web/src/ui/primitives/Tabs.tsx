// Tabs and the panel of the selected one (v4implementation D-181, WAI-ARIA
// tabs with manual activation). One tab is in the page's tab order, the
// selected one; ← → Home End move the focus between tabs and Enter or Space
// selects the focused tab. Switching shows the new panel at once, without
// motion (§17.4).
//
// While `disabled`, every tab is dimmed and none can be selected; the caller
// puts a placeholder in the panel.

import { type KeyboardEvent, type ReactNode, useId, useRef } from "react";

import styles from "../styles/controls.module.css";

export type TabItem<T extends string> = {
  id: T;
  label: string;
  icon?: ReactNode;
};

export type TabsProps<T extends string> = {
  tabs: readonly TabItem<T>[];
  selected: T;
  onSelect: (id: T) => void;
  /** The name of the tab list, for assistive technology. */
  label: string;
  disabled?: boolean;
  /** The panel of the selected tab. */
  children: ReactNode;
};

export function Tabs<T extends string>({ tabs, selected, onSelect, label, disabled = false, children }: TabsProps<T>) {
  const base = useId();
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const tabId = (id: T): string => `${base}-tab-${id}`;
  const panelId = `${base}-panel`;

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>): void => {
    const current = refs.current.findIndex((el) => el === document.activeElement);
    if (current < 0) {
      return;
    }
    const last = tabs.length - 1;
    let next: number;
    switch (event.key) {
      case "ArrowRight":
        next = current === last ? 0 : current + 1;
        break;
      case "ArrowLeft":
        next = current === 0 ? last : current - 1;
        break;
      case "Home":
        next = 0;
        break;
      case "End":
        next = last;
        break;
      default:
        return;
    }
    event.preventDefault();
    refs.current[next]?.focus();
  };

  return (
    <div className={styles.tabs}>
      <div role="tablist" aria-label={label} className={styles.tabList} onKeyDown={onKeyDown}>
        {tabs.map((tab, index) => {
          const isSelected = tab.id === selected;
          return (
            <button
              key={tab.id}
              ref={(el) => {
                refs.current[index] = el;
              }}
              type="button"
              role="tab"
              id={tabId(tab.id)}
              className={styles.tab}
              aria-selected={isSelected}
              aria-controls={panelId}
              aria-disabled={disabled ? true : undefined}
              tabIndex={isSelected ? 0 : -1}
              // A native button turns Enter and Space into a click.
              onClick={() => {
                if (!disabled && !isSelected) {
                  onSelect(tab.id);
                }
              }}
            >
              {tab.icon}
              <span className={styles.tabLabel}>{tab.label}</span>
            </button>
          );
        })}
      </div>
      <div
        role="tabpanel"
        id={panelId}
        aria-labelledby={tabId(selected)}
        aria-busy={disabled ? true : undefined}
        className={styles.tabPanel}
        tabIndex={0}
      >
        {children}
      </div>
    </div>
  );
}
