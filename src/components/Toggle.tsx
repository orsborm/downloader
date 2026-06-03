// 共享开关组件
// 统一所有 toggle/switch 的样式和行为

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export function Toggle({ checked, onChange, disabled = false }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => !disabled && onChange(!checked)}
      className={`
        relative inline-flex h-[22px] w-[40px] rounded-full p-[2px]
        transition-all duration-200 ease-in-out
        focus:outline-none focus:ring-2 focus:ring-accent/40 focus:ring-offset-1 focus:ring-offset-primary
        ${checked
          ? "bg-accent hover:bg-accent-hover"
          : "bg-border hover:bg-text-muted/30"
        }
        ${disabled ? "opacity-40 cursor-not-allowed" : "cursor-pointer"}
      `}
    >
      <span
        className={`
          block h-[18px] w-[18px] rounded-full bg-white shadow-sm
          transition-transform duration-200 ease-in-out
          ${checked ? "translate-x-[18px]" : "translate-x-0"}
        `}
      />
    </button>
  );
}
