import { useEffect, useRef } from 'react';

/**
 * useSpotlight V3.2 - "Extreme Performance"
 * 
 * 变更:
 * 1. 节流与阈值: 引入坐标偏差检查，只有位移 > 1px 时才触发更新。
 * 2. 局部优化: 仅处理 .spotlight-card 及其磁性子元素。
 */
export function useSpotlight() {
  const containerRef = useRef(null);
  const lastPos = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    let rafId = null;

    const handleMouseMove = (e) => {
      // 阈值检查：减少微型位移带来的冗余计算
      if (Math.abs(e.clientX - lastPos.current.x) < 1 && Math.abs(e.clientY - lastPos.current.y) < 1) {
        return;
      }
      
      const card = e.target.closest('.spotlight-card');
      if (!card) return;

      lastPos.current = { x: e.clientX, y: e.clientY };

      if (rafId) cancelAnimationFrame(rafId);

      rafId = requestAnimationFrame(() => {
        const rect = card.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        
        card.style.setProperty('--mouse-x', `${x}px`);
        card.style.setProperty('--mouse-y', `${y}px`);

        // 磁性偏移计算
        const centerX = rect.width / 2;
        const centerY = rect.height / 2;
        const percentX = (x - centerX) / centerX;
        const percentY = (y - centerY) / centerY;

        const strength = 4;
        card.style.setProperty('--mag-x', `${(percentX * strength).toFixed(2)}px`);
        card.style.setProperty('--mag-y', `${(percentY * strength).toFixed(2)}px`);
      });
    };

    container.addEventListener('mousemove', handleMouseMove, { passive: true });
    
    return () => {
      container.removeEventListener('mousemove', handleMouseMove);
      if (rafId) cancelAnimationFrame(rafId);
    };
  }, []);

  return containerRef;
}
