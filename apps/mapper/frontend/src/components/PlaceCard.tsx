import { useEffect, useState } from 'react';
import type { SelectedPlace } from '../map/usePlaceSelect';

const CATEGORY_ICONS: Record<string, string> = {
    restaurant: '🍽️', fast_food: '🍔', food_court: '🍽️', cafe: '☕', bar: '🍸',
    pub: '🍺', biergarten: '🍺', bakery: '🥖', ice_cream: '🍦',
    hotel: '🛏️', hostel: '🛏️', guest_house: '🛏️', motel: '🛏️',
    bank: '🏦', atm: '🏧', post_office: '📮', hospital: '🏥', clinic: '🏥',
    pharmacy: '💊', doctors: '🩺', dentist: '🦷',
    school: '🎓', university: '🎓', college: '🎓', kindergarten: '🧸',
    museum: '🏛️', gallery: '🖼️', theatre: '🎭', cinema: '🎬', library: '📚',
    park: '🌳', garden: '🌳', playground: '🛝', pitch: '⚽', stadium: '🏟️',
    sports_centre: '🏋️', swimming_pool: '🏊',
    supermarket: '🛒', convenience: '🏪', mall: '🛍️', marketplace: '🧺',
    clothes: '👕', books: '📚', florist: '💐', hairdresser: '💈',
    fuel: '⛽', charging_station: '🔌', parking: '🅿️', car_rental: '🚗',
    bus_stop: '🚌', bus_station: '🚌', station: '🚉', subway_entrance: '🚇',
    tram_stop: '🚊', ferry_terminal: '⛴️', taxi: '🚕',
    place_of_worship: '⛪', church: '⛪', mosque: '🕌', synagogue: '🕍', temple: '🛕',
    attraction: '📸', viewpoint: '🔭', artwork: '🎨', monument: '🗿',
    castle: '🏰', ruins: '🏚️', zoo: '🦁', aquarium: '🐟', theme_park: '🎡',
    police: '🚓', fire_station: '🚒', townhall: '🏛️', embassy: '🏛️',
    toilets: '🚻', drinking_water: '🚰', bench: '🪑',
};

const KIND_ICONS: Record<string, string> = {
    mountain_peak: '⛰️',
    aerodrome_label: '✈️',
    place: '📍',
    housenumber: '🏠',
    transportation_name: '🛣️',
};

function iconFor(place: SelectedPlace): string {
    return CATEGORY_ICONS[place.rawCategory] || KIND_ICONS[place.kind] || '📌';
}

const HEROES = [
    'linear-gradient(135deg,#38bdf8,#0ea5e9)',
    'linear-gradient(135deg,#34d399,#059669)',
    'linear-gradient(135deg,#fbbf24,#f59e0b)',
    'linear-gradient(135deg,#f472b6,#db2777)',
    'linear-gradient(135deg,#a78bfa,#7c3aed)',
];
function heroFor(place: SelectedPlace): string {
    let h = 0;
    for (const c of place.rawCategory || place.kind) h = (h * 31 + c.charCodeAt(0)) | 0;
    return HEROES[Math.abs(h) % HEROES.length];
}

interface Action {
    key: string;
    icon: string;
    label: string;
    run: () => void;
}

export default function PlaceCard({
    place,
    onClose,
}: {
    place: SelectedPlace;
    onClose: () => void;
}) {
    const [toast, setToast] = useState<string | null>(null);

    useEffect(() => {
        setToast(null);
    }, [place]);

    const flash = (msg: string) => {
        setToast(msg);
        window.setTimeout(() => setToast(m => (m === msg ? null : m)), 1500);
    };
    const coords = `${place.lat.toFixed(5)}, ${place.lng.toFixed(5)}`;
    const copy = async (text: string, label: string) => {
        try {
            await navigator.clipboard.writeText(text);
            flash(label);
        } catch {
            flash('Copy failed');
        }
    };

    const actions: Action[] = [
        { key: 'share', icon: '⇪', label: 'Share', run: () => copy(`${place.name} — ${coords}`, 'Link copied') },
    ];

    return (
        <div className="place-card" role="dialog" aria-label={place.name}>
            <div className="place-card-handle" aria-hidden="true" />
            <div className="place-card-hero" style={{ background: heroFor(place) }}>
                <span className="place-card-hero-icon">{iconFor(place)}</span>
                <button className="place-card-close" onClick={onClose} aria-label="Close">
                    ×
                </button>
            </div>

            <div className="place-card-body">
                <h2 className="place-card-title">{place.name}</h2>
                <div className="place-card-sub">
                    {place.category || 'Place'}
                    {place.approximate && <span className="place-card-approx"> · near here</span>}
                </div>

                <div className="place-card-actions">
                    {actions.map(a => (
                        <button key={a.key} className="pc-act" onClick={a.run}>
                            <span className="pc-act-ic">{a.icon}</span>
                            <span>{a.label}</span>
                        </button>
                    ))}
                </div>

                <div className="place-card-rows">
                    <button className="pc-row pc-row-btn" onClick={() => copy(coords, 'Coordinates copied')}>
                        <span className="pc-row-ic">📍</span>
                        <span className="pc-row-txt">{coords}</span>
                        <span className="pc-row-aside">Copy</span>
                    </button>
                    <div className="pc-row">
                        <span className="pc-row-ic">🏷️</span>
                        <span className="pc-row-txt">{place.category || place.kind}</span>
                    </div>
                </div>
            </div>

            {toast && <div className="place-card-toast">{toast}</div>}
        </div>
    );
}
