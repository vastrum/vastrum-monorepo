import React, { ReactNode } from 'react';

interface EmptyStateProps {
    icon: ReactNode;
    title: string;
    description?: string;
}

function EmptyState({ icon, title, description }: EmptyStateProps): React.JSX.Element {
    return (
        <div className="p-12 text-center">
            <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-app-bg-tertiary mb-4">
                {icon}
            </div>
            <h3 className="text-xl font-semibold mb-2">{title}</h3>
            {description && (
                <p className="text-app-text-secondary">{description}</p>
            )}
        </div>
    );
}

export default EmptyState;
