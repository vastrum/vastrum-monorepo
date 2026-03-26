import { CloseIcon } from './Icons';
import { SLIPPAGE_PRESETS } from '../constants';
import type { SwapSettings } from '../types';

interface SettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
    settings: SwapSettings;
    onSettingsChange: (settings: SwapSettings) => void;
}

export function SettingsModal({ isOpen, onClose, settings, onSettingsChange }: SettingsModalProps) {
    const handleSlippageChange = (value: string) => {
        if (value === '' || /^\d*\.?\d*$/.test(value)) {
            const num = parseFloat(value);
            if (value === '' || (num >= 0 && num <= 50)) {
                onSettingsChange({ ...settings, slippage: value });
            }
        }
    };

    const handleDeadlineChange = (value: string) => {
        if (value === '' || /^\d+$/.test(value)) {
            const num = parseInt(value);
            if (value === '' || (num >= 1 && num <= 180)) {
                onSettingsChange({ ...settings, deadline: value });
            }
        }
    };

    const slippageNum = parseFloat(settings.slippage) || 0;
    const isHighSlippage = slippageNum > 5;
    const isLowSlippage = slippageNum < 0.1 && slippageNum > 0;

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
            <div
                className="absolute inset-0 bg-black/60 backdrop-blur-sm"
                onClick={onClose}
            />

            <div className="relative bg-app-bg-secondary border border-app-border rounded-2xl w-full max-w-[400px] mx-4 p-4">
                <div className="flex justify-between items-center mb-4">
                    <h2 className="text-app-text-primary text-lg font-semibold">Settings</h2>
                    <button
                        onClick={onClose}
                        className="text-app-text-secondary hover:text-app-text-primary transition-colors p-1"
                    >
                        <CloseIcon />
                    </button>
                </div>

                <div className="mb-4">
                    <div className="text-app-text-secondary text-sm mb-2">Max Slippage</div>
                    <div className="flex gap-2">
                        {SLIPPAGE_PRESETS.map((preset) => (
                            <button
                                key={preset}
                                onClick={() => onSettingsChange({ ...settings, slippage: preset })}
                                className={`px-3 py-2 rounded-xl text-sm font-medium transition-colors ${settings.slippage === preset
                                    ? 'bg-app-accent-blue text-white'
                                    : 'bg-app-bg-tertiary text-app-text-primary hover:bg-app-hover'
                                    }`}
                            >
                                {preset}%
                            </button>
                        ))}
                        <div className="flex-1 relative">
                            <input
                                type="text"
                                inputMode="decimal"
                                value={settings.slippage}
                                onChange={(e) => handleSlippageChange(e.target.value)}
                                placeholder="Custom"
                                className={`w-full bg-app-bg-tertiary border rounded-xl py-2 px-3 pr-8 text-app-text-primary text-sm outline-none transition-colors ${!SLIPPAGE_PRESETS.includes(settings.slippage)
                                    ? 'border-app-accent-blue'
                                    : 'border-app-border focus:border-app-accent-blue'
                                    }`}
                            />
                            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-app-text-secondary text-sm">
                                %
                            </span>
                        </div>
                    </div>
                    {isHighSlippage && (
                        <div className="mt-2 text-app-accent-orange text-xs">
                            High slippage increases risk of unfavorable trades
                        </div>
                    )}
                    {isLowSlippage && (
                        <div className="mt-2 text-app-accent-orange text-xs">
                            Low slippage may cause transaction to fail
                        </div>
                    )}
                </div>

                <div>
                    <div className="text-app-text-secondary text-sm mb-2">Transaction Deadline</div>
                    <div className="flex items-center gap-2">
                        <input
                            type="text"
                            inputMode="numeric"
                            value={settings.deadline}
                            onChange={(e) => handleDeadlineChange(e.target.value)}
                            className="w-20 bg-app-bg-tertiary border border-app-border rounded-xl py-2 px-3 text-app-text-primary text-sm outline-none focus:border-app-accent-blue transition-colors"
                        />
                        <span className="text-app-text-secondary text-sm">minutes</span>
                    </div>
                </div>
            </div>
        </div>
    );
}
