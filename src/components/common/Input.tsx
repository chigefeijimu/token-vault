import { InputHTMLAttributes, forwardRef } from 'react';
import clsx from 'clsx';

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  hint?: string;
  leftElement?: React.ReactNode;
  rightElement?: React.ReactNode;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(({
  label,
  error,
  hint,
  leftElement,
  rightElement,
  className,
  ...props
}, ref) => {
  return (
    <div className="w-full">
      {label && (
        <label className="block text-sm font-medium text-vault-text-secondary mb-1.5">
          {label}
        </label>
      )}
      <div className="relative">
        {leftElement && (
          <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-vault-text-secondary">
            {leftElement}
          </div>
        )}
        <input
          ref={ref}
          className={clsx(
            'w-full bg-vault-bg border border-vault-border rounded-lg px-4 py-2.5 text-vault-text placeholder:text-gray-500 transition-all duration-150',
            'focus:border-transparent focus:ring-2 focus:ring-vault-gradient/50',
            error && 'border-red-500 focus:ring-red-500/50',
            leftElement && 'pl-10',
            rightElement && 'pr-10',
            className
          )}
          {...props}
        />
        {rightElement && (
          <div className="absolute inset-y-0 right-0 pr-3 flex items-center">
            {rightElement}
          </div>
        )}
      </div>
      {error && (
        <p className="mt-1.5 text-sm text-red-400">{error}</p>
      )}
      {hint && !error && (
        <p className="mt-1.5 text-sm text-vault-text-secondary">{hint}</p>
      )}
    </div>
  );
});

Input.displayName = 'Input';
