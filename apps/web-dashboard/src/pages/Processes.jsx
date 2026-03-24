import React, { useState, useEffect } from 'react';
import { useSpotlight } from '../hooks/useSpotlight';
import { useTranslation } from 'react-i18next';
import { 
  Cpu, 
  Activity, 
  Clock, 
  Trash2, 
  RefreshCw, 
  Settings2, 
  User, 
  Globe, 
  X, 
  Check, 
  Zap,
  ShieldCheck,
  AlertTriangle
} from 'lucide-react';
import useProcessStore from '../store/useProcessStore';
import useAppStore from '../store/useAppStore';
import { BaseModal, ConfirmModal } from '../components/Modal';

const ConfigModal = ({ isOpen, onClose }) => {
  const { t } = useTranslation();
  const { config, updateConfig } = useProcessStore();
  const [maxInstances, setMaxInstances] = useState(config.max_instances);
  const [idleTimeout, setIdleTimeout] = useState(config.idle_timeout_secs);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setMaxInstances(config.max_instances);
      setIdleTimeout(config.idle_timeout_secs);
    }
  }, [isOpen, config]);

  const handleSave = async (e) => {
    e.preventDefault();
    setIsSaving(true);
    const { addToast } = useAppStore();
    try {
      await updateConfig({
        max_instances: parseInt(maxInstances),
        idle_timeout_secs: parseInt(idleTimeout),
      });
      addToast(t('settings.syncSuccess') || 'Config updated', 'success');
      onClose();
    } catch (err) {
      addToast(t('settings.syncFailed') + err.message, 'error');
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <BaseModal 
      isOpen={isOpen} 
      onClose={onClose} 
      title={t('processes.modalTitle')} 
      subtitle={t('processes.modalSubtitle')} 
      accentColor="bg-blue-600" 
      footerText={t('common.close')}
    >
      <form onSubmit={handleSave} className="py-6 space-y-6">
        <div className="flex flex-col gap-2">
          <label className="text-[10px] font-black text-foreground/20 uppercase tracking-[0.3em] pl-1">{t('processes.maxInstances')}</label>
          <div className="flex items-center gap-4 bg-foreground/[0.03] border border-glass-border rounded-2xl px-6 py-4">
            <Cpu className="w-4 h-4 text-foreground/20" />
            <input 
              type="number" 
              value={maxInstances}
              onChange={(e) => setMaxInstances(e.target.value)}
              className="flex-1 bg-transparent text-sm text-foreground focus:outline-none" 
            />
            <span className="text-[10px] font-black text-foreground/10 uppercase italic">{t('processes.slots')}</span>
          </div>
        </div>

        <div className="flex flex-col gap-2">
          <label className="text-[10px] font-black text-foreground/20 uppercase tracking-[0.3em] pl-1">{t('processes.idleTimeout')}</label>
          <div className="flex items-center gap-4 bg-foreground/[0.03] border border-glass-border rounded-2xl px-6 py-4">
            <Clock className="w-4 h-4 text-foreground/20" />
            <input 
              type="number" 
              value={idleTimeout}
              onChange={(e) => setIdleTimeout(e.target.value)}
              className="flex-1 bg-transparent text-sm text-foreground focus:outline-none" 
            />
            <span className="text-[10px] font-black text-foreground/10 uppercase italic">Seconds</span>
          </div>
          <p className="text-[9px] text-foreground/20 italic pl-2 mt-1">
            {t('processes.idleTimeoutDesc')}
          </p>
        </div>

        <button 
          type="submit"
          disabled={isSaving}
          className="w-full py-4 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white text-[11px] font-black uppercase tracking-[0.3em] rounded-2xl transition-all shadow-[0_0_30px_rgba(37,99,235,0.2)] flex items-center justify-center gap-3"
        >
          {isSaving ? <RefreshCw className="w-4 h-4 animate-spin" /> : <ShieldCheck className="w-4 h-4" />}
          {t('processes.deployBtn')}
        </button>
      </form>
    </BaseModal>
  );
};

const ProcessRow = ({ process }) => {
  const { t } = useTranslation();
  const { killProcess } = useProcessStore();
  const { addToast } = useAppStore();
  const [isDeleting, setIsDeleting] = useState(false);
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const isOrphan = process.status === 'orphan';

  const handleKill = async () => {
    setIsConfirmOpen(false);
    setIsDeleting(true);
    try {
      await killProcess(process.id);
      addToast(t('processes.killSuccess') || 'Process terminated', 'success');
    } catch (err) {
      addToast(t('monitor.errorTrace') + ': ' + err.message, 'error');
      setIsDeleting(false);
    }
  };

  const confirmMsg = isOrphan
    ? t('processes.orphanMsg', { id: process.id })
    : t('processes.killMsg', { id: process.id });

  const formatUptime = (secs) => {
    const now = Math.floor(Date.now() / 1000);
    const diff = Math.max(0, now - secs);
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ${diff % 60}s`;
    return `${Math.floor(diff / 3600)}h ${Math.floor((diff % 3600) / 60)}m`;
  };

  const formatLastActive = (secs) => {
    const now = Math.floor(Date.now() / 1000);
    const diff = Math.max(0, now - secs);
    if (diff < 5) return t('processes.activeNow');
    const mins = Math.floor(diff / 60);
    if (mins < 1) return t('processes.activePrev', { count: diff });
    return t('processes.activePrev', { count: mins });
  };

  if (isOrphan) {
    return (
      <div className="group spotlight-card flex items-center px-4 py-3 bg-foreground/[0.01] hover:bg-foreground/[0.02] border-b border-glass-border transition-all cursor-default opacity-40 grayscale relative overflow-hidden">
        <div className="grain-overlay" />
        <div className="flex items-center gap-4 w-[25%] min-w-[200px]">
          <div className="shrink-0 p-2 rounded-lg bg-foreground/5 text-foreground/20 border border-glass-border">
            <Activity className="w-4 h-4" />
          </div>
          <div className="flex flex-col min-w-0 flex-1">
            <span className="text-[13px] md:text-sm font-black text-foreground/40 truncate italic tracking-tight">{process.id}</span>
            <span className="text-[9px] font-black text-foreground/20 uppercase tracking-[0.2em]">ORPHAN · PROCESS OFFLINE</span>
          </div>
        </div>
        
        <div className="w-[30%] px-6 flex flex-col gap-0.5">
          <div className="flex items-center gap-2">
            <User className="w-3 h-3 text-foreground/10" />
            <span className="text-[11px] font-bold text-foreground/20 truncate uppercase">— {t('processes.orphan')} —</span>
          </div>
          <div className="flex items-center gap-2 pl-5">
            <span className="text-[9px] font-black text-foreground/10 uppercase tracking-widest">{formatLastActive(process.last_accessed_secs)}</span>
          </div>
        </div>

        <div className="w-[20%] flex flex-col items-center gap-1 shrink-0 px-4">
          <div className="flex items-center gap-1.5 text-[9px] font-black tracking-widest text-foreground/20 uppercase">
            <AlertTriangle className="w-3 h-3 text-amber-500/30" />
            <span>{t('processes.legacy')}</span>
          </div>
          <div className="text-[10px] text-foreground/30 font-mono font-black tracking-widest">
            {formatUptime(process.created_at_secs)}
          </div>
        </div>

        <div className="flex-1 flex items-center justify-end gap-2 shrink-0 pr-4">
          <button 
            onClick={() => setIsConfirmOpen(true)} 
            disabled={isDeleting}
            className="px-4 py-1.5 bg-foreground/[0.03] hover:bg-amber-500/10 border border-glass-border hover:border-amber-500/20 text-foreground/20 hover:text-amber-400 rounded-lg text-[9px] font-black uppercase tracking-[0.2em] transition-all disabled:opacity-30 flex items-center gap-2 group/btn"
          >
            {isDeleting ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Trash2 className="w-3 h-3 group-hover/btn:scale-110 transition-transform" />}
            {t('processes.cleanBtn')}
          </button>
        </div>

        <ConfirmModal 
          isOpen={isConfirmOpen}
          onClose={() => setIsConfirmOpen(false)}
          onConfirm={handleKill}
          title={isOrphan ? t('processes.confirmTitleClean') : t('processes.confirmTitleKill')}
          message={confirmMsg}
          type={isOrphan ? "warning" : "danger"}
          confirmText={isOrphan ? t('processes.cleanBtn') : t('processes.killBtn')}
        />
      </div>
    );
  }

  return (
    <div className="group spotlight-card flex items-center px-4 py-3 bg-foreground/[0.01] hover:bg-foreground/[0.03] border-b border-glass-border transition-all cursor-default relative overflow-hidden">
      <div className="grain-overlay" />
      <div className="flex items-center gap-4 w-[25%] min-w-[200px]">
        <div className="shrink-0 p-2 rounded-lg bg-teal-500/10 text-teal-500 border border-teal-500/20">
          <Activity className="w-4 h-4 animate-pulse" />
        </div>
        <div className="flex flex-col min-w-0 flex-1">
          <span className="text-[13px] md:text-sm font-black text-foreground/90 truncate italic tracking-tight">{process.id}</span>
          <span className="text-[9px] font-black text-teal-500/50 uppercase tracking-[0.2em]">{process.grpc_addr}</span>
        </div>
      </div>
      
      <div className="w-[30%] px-6 flex flex-col gap-0.5">
        <div className="flex items-center gap-2">
          <User className="w-3 h-3 text-foreground/20" />
          <span className="text-[11px] font-bold text-foreground/60 truncate uppercase">{process.identity}</span>
        </div>
        <div className="flex items-center gap-2 pl-5">
           <span className="text-[9px] font-black text-foreground/10 uppercase tracking-widest">{formatLastActive(process.last_accessed_secs)}</span>
        </div>
      </div>

      <div className="w-[20%] flex flex-col items-center gap-1 shrink-0 px-4">
        <div className="flex items-center gap-1.5 text-[9px] font-black tracking-widest text-foreground/30 uppercase">
          <Zap className="w-3 h-3 text-amber-500/50" />
          <span>{t('processes.uptime')}</span>
        </div>
        <div className="text-[10px] text-foreground/80 font-mono font-black tracking-widest">
          {formatUptime(process.created_at_secs)}
        </div>
      </div>

      <div className="flex-1 flex items-center justify-end gap-2 shrink-0 pr-4">
        <button 
          onClick={() => setIsConfirmOpen(true)} 
          disabled={isDeleting}
          className="px-4 py-1.5 bg-foreground/[0.03] hover:bg-rose-500/20 border border-glass-border hover:border-rose-500/30 text-foreground/20 hover:text-rose-500 rounded-lg text-[9px] font-black uppercase tracking-[0.2em] transition-all disabled:opacity-30 flex items-center gap-2 group/btn"
        >
          {isDeleting ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Trash2 className="w-3 h-3 group-hover/btn:scale-110 transition-transform" />}
          {t('processes.killBtn')}
        </button>
      </div>

      <ConfirmModal 
        isOpen={isConfirmOpen}
        onClose={() => setIsConfirmOpen(false)}
        onConfirm={handleKill}
        title={isOrphan ? t('processes.confirmTitleClean') : t('processes.confirmTitleKill')}
        message={confirmMsg}
        type={isOrphan ? "warning" : "danger"}
        confirmText={isOrphan ? t('processes.cleanBtn') : t('processes.killBtn')}
      />
    </div>
  );
};


const Processes = () => {
  const { t } = useTranslation();
  const spotlightRef = useSpotlight();
  const { processes, config, fetchProcesses, fetchConfig, isLoading } = useProcessStore();
  const [isConfigOpen, setIsConfigOpen] = useState(false);

  useEffect(() => {
    fetchProcesses();
    fetchConfig();
    const timer = setInterval(() => fetchProcesses(), 10000); // 10秒自动刷新一次
    return () => clearInterval(timer);
  }, []);

  return (
    <div ref={spotlightRef} className="space-y-8 fade-in relative spotlight-group">
      <div className="sticky top-0 z-[100] -mx-4 px-4 py-4 mb-8 bg-background/60 backdrop-blur-2xl border-b border-glass-border flex flex-col md:flex-row justify-between items-center gap-4 transition-all duration-300">
        <div className="flex items-center gap-8 px-4">
          <div className="flex flex-col">
            <span className="text-[10px] font-black text-foreground/20 uppercase tracking-[0.3em]">{t('processes.title')}</span>
            <div className="flex items-baseline gap-2">
               <span className="text-xl font-black italic tracking-tight text-highlight">{processes.length}</span>
               <span className="text-sm font-black text-foreground/10 italic">/</span>
               <span className="text-sm font-black text-foreground/40 italic">{config.max_instances} {t('processes.slots')}</span>
            </div>
          </div>
          
          <div className="h-8 w-px bg-foreground/10 hidden md:block"></div>
          
          <div className="flex flex-col">
            <span className="text-[10px] font-black text-foreground/20 uppercase tracking-[0.3em]">{t('processes.strategy')}</span>
            <div className="flex items-center gap-3 text-[10px] font-bold text-teal-400 mt-1">
              <span className="flex items-center gap-1"><Check className="w-3 h-3" /> {t('processes.autoRecycle')}</span>
              <span className="w-1 h-1 rounded-full bg-foreground/10"></span>
              <span className="text-foreground/40 uppercase tracking-tighter">TTL: {config.idle_timeout_secs}s</span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2 pr-4">
          <button 
            onClick={() => fetchProcesses()} 
            className={`w-10 h-10 flex items-center justify-center bg-foreground/[0.03] border border-glass-border rounded-xl transition-all ${isLoading ? 'text-teal-400' : 'text-foreground/40 hover:text-foreground'}`}
            title={t('processes.manualSync')}
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          </button>
          
          <button 
            onClick={() => setIsConfigOpen(true)}
            className="h-10 px-6 bg-foreground/[0.03] border border-glass-border text-foreground hover:bg-foreground hover:text-background rounded-xl font-black text-[11px] uppercase tracking-widest transition-all flex items-center gap-2 group"
          >
            <Settings2 className="w-4 h-4 group-hover:rotate-90 transition-transform duration-500" />
            {t('processes.configBtn')}
          </button>
        </div>
      </div>

      <div className="px-4">
        <div className="bg-foreground/[0.02] border border-glass-border rounded-[2rem] overflow-hidden shadow-2xl relative">
          <div className="flex items-center px-6 py-4 bg-foreground/[0.01] border-b border-glass-border text-[10px] font-black uppercase tracking-[0.2em] text-foreground/40">
            <div className="w-[25%] min-w-[200px]">{t('processes.tableID')}</div>
            <div className="w-[30%] px-6">{t('processes.tableAccount')}</div>
            <div className="w-[20%] text-center px-4">{t('processes.tableUptime')}</div>
            <div className="flex-1 text-right pr-6">{t('processes.tableOps')}</div>
          </div>

          <div className="divide-y divide-white/[0.02]">
            {isLoading && processes.length === 0 ? (
              <div className="flex-1 flex flex-col items-center justify-center py-32 gap-6 opacity-30">
                <RefreshCw className="w-12 h-12 animate-spin text-teal-500" />
                <span className="text-[11px] font-black uppercase tracking-[0.5em] animate-pulse">{t('processes.loading')}</span>
              </div>
            ) : processes.length > 0 ? (
              processes.map(p => <ProcessRow key={p.id} process={p} />)
            ) : (
              <div className="flex-1 flex flex-col items-center justify-center py-32 gap-6 opacity-10 grayscale">
                <div className="p-6 rounded-full bg-foreground/5">
                  <Activity className="w-8 h-8 text-foreground/5" />
                </div>
                <div className="text-center">
                  <div className="text-[11px] font-black uppercase tracking-[0.4em] text-foreground/20 mb-1">{t('processes.empty')}</div>
                  <div className="text-[9px] font-bold text-foreground/10 uppercase italic">NO ACTIVE LS INSTANCES DETECTED</div>
                </div>
              </div>
            )}
          </div>
          
          <div className="px-8 py-4 bg-foreground/[0.01] border-t border-glass-border flex justify-between items-center text-[9px] font-black text-foreground/10 tracking-widest uppercase italic">
            <div className="flex items-center gap-4">
              <span className="flex items-center gap-2"><Globe className="w-3 h-3" /> ORCHESTRATOR BRIDGE ACTIVE</span>
              <span className="flex items-center gap-2 text-teal-500/20"><ShieldCheck className="w-3 h-3 text-teal-500/50" /> {t('processes.isolated')}</span>
            </div>
            <div>ANTIGRAVITY_INSTANCE_MANAGEMENT_SERVICE</div>
          </div>
        </div>
      </div>

      <ConfigModal isOpen={isConfigOpen} onClose={() => setIsConfigOpen(false)} />
    </div>
  );
};

export default Processes;
