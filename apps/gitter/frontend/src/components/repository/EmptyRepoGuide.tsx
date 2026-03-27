import React from 'react';
import { Terminal, Download } from 'lucide-react';

interface EmptyRepoGuideProps {
    repositoryName: string;
}

const EmptyRepoGuide = ({ repositoryName }: EmptyRepoGuideProps): React.JSX.Element => {
    return (
        <div className="p-6 space-y-6">
            <p className="text-sm text-app-text-secondary">
                This repository is empty. If you are the owner of the repo you can push to it using vastrum-cli.
            </p>
            <p className="text-sm text-app-text-secondary">
                If you are the owner you will need to pass your private key, you can get it from the wallet modal opened by the button at the center of the top of the page. There is also a video tutorial at{' '}
                <a href="https://docs.vastrum.net/apps/gitter" target="_blank" rel="noopener noreferrer" className="text-app-accent-blue hover:underline">docs.vastrum.net/apps/gitter</a>
            </p>

            {/* Step 1: Install CLI */}
            <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
                <div className="flex items-start gap-3 mb-3">
                    <div className="w-6 h-6 rounded-full bg-app-accent-blue text-white flex items-center justify-center text-sm font-semibold flex-shrink-0">
                        1
                    </div>
                    <div className="flex-1">
                        <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                            <Download className="w-4 h-4" />
                            Install vastrum-cli
                        </h3>
                        <div className="bg-app-bg-primary border border-app-border rounded px-3 py-2 font-mono text-xs text-app-accent-green break-all">
                            curl -sSf https://raw.githubusercontent.com/vastrum/vastrum-monorepo/HEAD/tooling/cli/install.sh | sh
                        </div>
                    </div>
                </div>
            </div>

            {/* Step 2: Push a new repo */}
            <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
                <div className="flex items-start gap-3">
                    <div className="w-6 h-6 rounded-full bg-app-accent-blue text-white flex items-center justify-center text-sm font-semibold flex-shrink-0">
                        2
                    </div>
                    <div className="flex-1">
                        <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                            <Terminal className="w-4 h-4" />
                            Push a new repository
                        </h3>
                        <div className="bg-app-bg-primary border border-app-border rounded px-3 py-2 font-mono text-xs text-app-accent-green space-y-1">
                            <div>git init</div>
                            <div>git add .</div>
                            <div>git commit -m "first commit"</div>
                            <div>vastrum-cli vastrum-git-push {repositoryName} &lt;your-private-key&gt;</div>
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
                    <div className="flex-1">
                        <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                            <Terminal className="w-4 h-4" />
                            Or push an existing repository
                        </h3>
                        <div className="bg-app-bg-primary border border-app-border rounded px-3 py-2 font-mono text-xs text-app-accent-green">
                            vastrum-cli vastrum-git-push {repositoryName} &lt;your-private-key&gt;
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default EmptyRepoGuide;
