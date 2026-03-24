import React from 'react';
import { createPortal } from 'react-dom';
import { X, AlertTriangle, HelpCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';

/**
 * 基础弹窗组件 - 承载容器
 */
export const BaseModal = ({ 
  isOpen, 
  onClose, 
  title, 
  subtitle, 
  footerText, 
  children, 
  borderColor = "glass-border", 
  accentColor = "bg-blue-500", 
  maxWidth = "max-w-xl",
  showFooter = true
}) => {
  const { t } = useTranslation();
  if (!isOpen) return null;

  const content = (
    <div className="fixed inset-0 z-[1000] flex items-center justify-center p-4 lg:p-12 animate-in fade-in duration-300">
      <div className="absolute inset-0 bg-background/80 backdrop-blur-2xl" onClick={onClose}></div>
      
      <div 
        className={`bg-background border border-${borderColor} w-full ${maxWidth} max-h-[85vh] rounded-[2.5rem] shadow-[0_0_100px_rgba(0,0,0,0.9)] relative z-10 flex flex-col overflow-hidden animate-in zoom-in slide-in-from-bottom-8 duration-500`}
      >
        {/* Header */}
        <div className="px-8 py-8 flex justify-between items-center bg-gradient-to-b from-foreground/[0.02] to-transparent shrink-0">
          <div className="flex flex-col">
            <h3 className="text-xl font-black tracking-tight text-foreground uppercase italic flex items-center gap-3">
              <div className={`w-1 h-5 ${accentColor} rounded-full shadow-[0_0_15px_rgba(59,130,246,0.6)]`}></div>
              {title}
            </h3>
            {subtitle && <span className="text-[10px] font-bold text-foreground/10 tracking-[0.4em] uppercase pl-4 truncate max-w-[300px] md:max-w-none">{subtitle}</span>}
          </div>
          <button 
            onClick={onClose} 
            className="w-10 h-10 flex items-center justify-center hover:bg-foreground/5 rounded-full transition-all text-foreground/10 hover:text-foreground border border-glass-border"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto px-8 pb-4 custom-scrollbar">
          {children}
        </div>

        {/* Footer */}
        {showFooter && (
          <div className="px-8 py-6 border-t border-glass-border flex justify-center shrink-0">
            <button 
              onClick={onClose} 
              className="text-[9px] font-black uppercase tracking-[0.6em] text-foreground/10 hover:text-foreground transition-all py-2 px-4"
            >
              {footerText || t('common.close')}
            </button>
          </div>
        )}
      </div>
    </div>
  );

  return createPortal(content, document.body);
};

/**
 * 确认弹窗组件 - 替代原生 confirm
 */
export const ConfirmModal = ({
  isOpen,
  onClose,
  onConfirm,
  title,
  message,
  confirmText,
  cancelText,
  type = "warning", // "warning" | "danger" | "info"
  isLoading = false
}) => {
  const { t } = useTranslation();
  const themes = {
    danger: {
      border: "rose-500/20",
      accent: "bg-rose-500",
      icon: <AlertTriangle className="w-8 h-8 text-rose-500" />,
      btn: "bg-rose-600 hover:bg-rose-500 shadow-[0_0_20px_rgba(244,63,94,0.2)]"
    },
    warning: {
      border: "amber-500/20",
      accent: "bg-amber-500",
      icon: <AlertTriangle className="w-8 h-8 text-amber-500" />,
      btn: "bg-amber-600 hover:bg-amber-500 shadow-[0_0_20px_rgba(245,158,11,0.2)]"
    },
    info: {
      border: "blue-500/20",
      accent: "bg-blue-500",
      icon: <HelpCircle className="w-8 h-8 text-blue-500" />,
      btn: "bg-blue-600 hover:bg-blue-500 shadow-[0_0_20px_rgba(59,130,246,0.2)]"
    }
  };

  const currentTheme = themes[type] || themes.warning;

  return (
    <BaseModal
      isOpen={isOpen}
      onClose={onClose}
      title={title || t('common.confirmTitle')}
      borderColor={currentTheme.border}
      accentColor={currentTheme.accent}
      maxWidth="max-w-md"
      showFooter={false}
    >
      <div className="py-6 flex flex-col items-center text-center gap-6">
        <div className="p-4 rounded-3xl bg-foreground/[0.02] border border-glass-border">
          {currentTheme.icon}
        </div>
        
        <div className="space-y-2">
          <p className="text-foreground/80 text-sm font-bold leading-relaxed px-4">
            {message}
          </p>
          <p className="text-[10px] text-foreground/20 font-black uppercase tracking-widest italic">
            {t('common.irreversibleMsg')}
          </p>
        </div>

        <div className="flex w-full gap-3 mt-4">
          <button
            onClick={onClose}
            className="flex-1 py-4 rounded-2xl bg-foreground/5 border border-glass-border text-foreground/40 text-[11px] font-black uppercase tracking-[0.2em] hover:bg-foreground/10 hover:text-foreground transition-all"
          >
            {cancelText || t('common.confirmCancel')}
          </button>
          <button
            onClick={onConfirm}
            disabled={isLoading}
            className={`flex-1 py-4 rounded-2xl ${currentTheme.btn} text-background text-[11px] font-black uppercase tracking-[0.2em] transition-all disabled:opacity-50`}
          >
            {isLoading ? t('common.processing') : (confirmText || t('common.confirmOk'))}
          </button>
        </div>
      </div>
    </BaseModal>
  );
};
