import React from 'react';
import { CheckCircle2, AlertCircle, Info, X, AlertTriangle } from 'lucide-react';
import useAppStore from '../store/useAppStore';

const Toast = ({ toast }) => {
  const { removeToast } = useAppStore();

  const getIcon = () => {
    switch (toast.type) {
      case 'success': return <CheckCircle2 className="w-4 h-4 text-emerald-500" />;
      case 'error': return <AlertCircle className="w-4 h-4 text-rose-500" />;
      case 'warning': return <AlertTriangle className="w-4 h-4 text-amber-500" />;
      default: return <Info className="w-4 h-4 text-blue-500" />;
    }
  };

  return (
    <div className="flex items-center gap-3 px-4 py-3 rounded-xl glass-card border-foreground/10 shadow-2xl animate-fade-in pointer-events-auto min-w-[300px] max-w-sm">
      <div className="shrink-0">{getIcon()}</div>
      <div className="flex-1 text-[13px] font-medium text-foreground opacity-90 leading-tight">
        {toast.message}
      </div>
      <button 
        onClick={() => removeToast(toast.id)}
        className="w-6 h-6 rounded-full flex items-center justify-center hover:bg-foreground/5 transition-colors opacity-40 hover:opacity-100"
      >
        <X className="w-3 h-3" />
      </button>
    </div>
  );
};

export const ToastContainer = () => {
  const { toasts } = useAppStore();

  return (
    <div className="fixed bottom-8 right-8 z-[100] flex flex-col gap-3 pointer-events-none">
      {toasts.map((toast) => (
        <Toast key={toast.id} toast={toast} />
      ))}
    </div>
  );
};

export default Toast;
