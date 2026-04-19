import React from 'react';
import { Loader2, CheckCircle2, XCircle } from 'lucide-react';
import type { Transaction } from '../types/wallet';

type TransactionStatus = Transaction['status'];

interface TransactionStatusBadgeProps {
  status: TransactionStatus;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
  className?: string;
}

const statusConfig: Record<TransactionStatus, {
  label: string;
  icon: React.ReactNode;
  containerClass: string;
  textClass: string;
}> = {
  pending: {
    label: 'Pending',
    icon: <Loader2 className="animate-spin" />,
    containerClass: 'bg-yellow-500/10 border-yellow-500/30 text-yellow-500',
    textClass: 'text-yellow-500',
  },
  confirmed: {
    label: 'Confirmed',
    icon: <CheckCircle2 />,
    containerClass: 'bg-green-500/10 border-green-500/30 text-green-500',
    textClass: 'text-green-500',
  },
  failed: {
    label: 'Failed',
    icon: <XCircle />,
    containerClass: 'bg-red-500/10 border-red-500/30 text-red-500',
    textClass: 'text-red-500',
  },
};

const sizeClasses = {
  sm: {
    container: 'px-2 py-0.5 text-xs gap-1',
    icon: 'w-3 h-3',
  },
  md: {
    container: 'px-2.5 py-1 text-sm gap-1.5',
    icon: 'w-4 h-4',
  },
  lg: {
    container: 'px-3 py-1.5 text-base gap-2',
    icon: 'w-5 h-5',
  },
};

export const TransactionStatusBadge: React.FC<TransactionStatusBadgeProps> = ({
  status,
  size = 'md',
  showLabel = true,
  className = '',
}) => {
  const config = statusConfig[status];
  const sizeConfig = sizeClasses[size];

  return (
    <span
      className={`
        inline-flex items-center rounded-full border font-medium
        ${config.containerClass}
        ${sizeConfig.container}
        ${className}
      `}
    >
      <span className={`${config.textClass} ${sizeConfig.icon}`}>
        {config.icon}
      </span>
      {showLabel && (
        <span className={config.textClass}>{config.label}</span>
      )}
    </span>
  );
};

export default TransactionStatusBadge;
