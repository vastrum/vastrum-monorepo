import React from 'react';
import { Terminal, Key } from 'lucide-react';

interface EmptyRepoGuideProps {
    repositoryName: string;
}

const EmptyRepoGuide = ({ repositoryName }: EmptyRepoGuideProps): React.JSX.Element => {
    return (
        <div className="p-6 space-y-6">
            <p className="text-sm text-app-text-secondary">
                This repository is empty, you can push a repository to it.
            </p>
            <p className="text-sm text-app-text-secondary">
                You must first register your SSH public key in the repository settings (click the Settings button above).
            </p>

            {/* Step 1: Push a new repo */}
            <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
                <div className="flex items-start gap-3">
                    <div className="w-6 h-6 rounded-full bg-app-accent-blue text-white flex items-center justify-center text-sm font-semibold flex-shrink-0">
                        1
                    </div>
                    <div className="flex-1">
                        <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                            <Key className="w-4 h-4" />
                            Register your SSH key
                        </h3>
                        <p className="text-sm text-app-text-secondary mb-2">
                            Click <strong>Settings</strong> above and paste your SSH public key (e.g. from <code className="text-xs bg-app-bg-primary px-1 rounded">~/.ssh/id_ed25519.pub</code>).
                        </p>
                    </div>
                </div>
            </div>

            {/* Step 2: Push a new repo */}
            <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
                <div className="flex items-start gap-3">
                    <div className="w-6 h-6 rounded-full bg-app-accent-blue text-white flex items-center justify-center text-sm font-semibold flex-shrink-0">
                        2
                    </div>
                    <div className="flex-1 min-w-0">
                        <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                            <Terminal className="w-4 h-4" />
                            Push a new repository
                        </h3>
                        <div className="bg-app-bg-primary border border-app-border rounded px-3 py-2 font-mono text-xs text-app-accent-green space-y-1 overflow-x-auto">
                            <div className="whitespace-nowrap">git init</div>
                            <div className="whitespace-nowrap">git add .</div>
                            <div className="whitespace-nowrap">git commit -m "first commit"</div>
                            <div className="whitespace-nowrap">git remote add origin ssh://git@gitrelay.vastrum.org:2222/{repositoryName}</div>
                            <div className="whitespace-nowrap">git push origin HEAD</div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Step 3: Push existing repo */}
            <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
                <div className="flex items-start gap-3">
                    <div className="w-6 h-6 rounded-full bg-app-accent-blue text-white flex items-center justify-center text-sm font-semibold flex-shrink-0">
                        3
                    </div>
                    <div className="flex-1 min-w-0">
                        <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                            <Terminal className="w-4 h-4" />
                            Or push an existing repository
                        </h3>
                        <div className="bg-app-bg-primary border border-app-border rounded px-3 py-2 font-mono text-xs text-app-accent-green space-y-1 overflow-x-auto">
                            <div className="whitespace-nowrap">git remote add origin ssh://git@gitrelay.vastrum.org:2222/{repositoryName}</div>
                            <div className="whitespace-nowrap">git push origin HEAD</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default EmptyRepoGuide;
