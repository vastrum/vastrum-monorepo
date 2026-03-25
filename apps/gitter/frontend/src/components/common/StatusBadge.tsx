import React from 'react';
import { Circle, Check } from 'lucide-react';

type Status = 'open' | 'merged' | 'closed';

interface StatusBadgeProps {
    status: Status;
}

function StatusBadge({ status }: StatusBadgeProps): React.JSX.Element {
    const getBadgeDetails = () => {
        switch (status) {
            case 'open':
                return {
                    color: 'bg-app-accent-green',
                    icon: <Circle className="w-4 h-4 fill-current" />,
                    text: 'Open'
                };
            case 'merged':
                return {
                    color: 'bg-app-accent-purple',
                    icon: <Check className="w-4 h-4" />,
                    text: 'Merged'
                };
            case 'closed':
                return {
                    color: 'bg-app-accent-red',
                    icon: <Circle className="w-4 h-4" />,
                    text: 'Closed'
                };
        }
    };

    const badge = getBadgeDetails();

    return (
        <span className={`inline-flex items-center gap-2 ${badge.color} text-white px-3 py-1.5 rounded-full text-sm font-semibold`}>
            {badge.icon}
            {badge.text}
        </span>
    );
}

export default StatusBadge;
