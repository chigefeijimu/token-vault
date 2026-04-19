import { Fragment, ReactNode } from 'react';
import { Dialog, DialogPanel, DialogTitle, Transition, TransitionChild } from '@headlessui/react';
import { X } from 'lucide-react';
import clsx from 'clsx';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
}

export function Modal({ isOpen, onClose, title, children, size = 'md' }: ModalProps) {
  return (
    <Transition show={isOpen} as={Fragment}>
      <Dialog onClose={onClose} className="relative z-50">
        <TransitionChild
          as={Fragment}
          enter="ease-out duration-200"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-150"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-black/60 backdrop-blur-sm" />
        </TransitionChild>

        <div className="fixed inset-0 flex items-center justify-center p-4">
          <TransitionChild
            as={Fragment}
            enter="ease-out duration-200"
            enterFrom="opacity-0 scale-95"
            enterTo="opacity-100 scale-100"
            leave="ease-in duration-150"
            leaveFrom="opacity-100 scale-100"
            leaveTo="opacity-0 scale-95"
          >
            <DialogPanel 
              className={clsx(
                'w-full bg-vault-card border border-vault-border rounded-2xl shadow-xl',
                'max-h-[85vh] overflow-hidden flex flex-col',
                {
                  'max-w-sm': size === 'sm',
                  'max-w-md': size === 'md',
                  'max-w-lg': size === 'lg',
                  'max-w-2xl': size === 'xl',
                }
              )}
            >
              {title && (
                <div className="flex items-center justify-between px-6 py-4 border-b border-vault-border">
                  <DialogTitle className="text-lg font-semibold">{title}</DialogTitle>
                  <button
                    onClick={onClose}
                    className="p-1 rounded-lg hover:bg-vault-border transition-colors"
                  >
                    <X className="w-5 h-5" />
                  </button>
                </div>
              )}
              <div className="p-6 overflow-y-auto">
                {children}
              </div>
            </DialogPanel>
          </TransitionChild>
        </div>
      </Dialog>
    </Transition>
  );
}
