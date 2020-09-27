export function rough_duration(ms: number): string {
	if (ms === 0) {
		return "0s";
	} else if (ms < 1000) {
		return `${ms}ms`;
	} else {
		let s = Math.round(ms / 1000);
		if (s < 60) {
			return `${s}s`;
		}
		let mins = Math.round(s / 60);
		if (mins < 60) {
			return `${mins}mins`;
		}
		let hrs = Math.round(mins / 60);
		if (hrs < 60) {
			return `${hrs}hrs`;
		}
		let days = Math.round(hrs / 24 * 10) / 10;
		const one_year_days = 365.25;
		if (days < one_year_days) {
			return `${days} days`;
		}
		let years = Math.round(days / one_year_days * 10) / 10;
		return `${years} years`;
	}
}
