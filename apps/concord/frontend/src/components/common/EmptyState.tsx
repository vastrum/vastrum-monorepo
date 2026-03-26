import React, { type ReactNode } from 'react';

function EmptyState({ icon, message }: { icon: ReactNode; message: string }): React.JSX.Element {
    return (
        <div className="px-2 py-6 text-center text-dc-text-muted">
            {icon}
            <p className="text-sm">{message}</p>
        </div>
    );
}

export default EmptyState;
