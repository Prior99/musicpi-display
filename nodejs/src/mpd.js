import { spawn } from 'child_process';

const mpd = () => {
    return new Promise((resolve) => {
        const parts = [];
        const child = spawn('mpc', ['-h', 'auenland']);
        child.stdout.on('data', (data) => parts.push(data));
        child.on('close', (code) => {
            if(code === 0) {
                resolve(parts.join(''));
            } else {
                reject();
            }
        });
    });
};

function parseSongString(string) {
	const splitter = ' - ';
	const splitterIndex = string.indexOf(splitter)
	return {
		artist: string.substr(0, splitterIndex),
		song: string.substr(splitterIndex + splitter.length)
	};
}

const statusRegex = /\[(\w+)\]\s+#(\d+)\/(\d+)\s+(\d+:\d+)\/(\d+:\d+)\s+\((\d+)\%\)/;
function parseStatusString(string) {
	const result = statusRegex.exec(string);
	return {
		playing: result[1] === 'playing',
		track: parseInt(result[2]),
		tracks: parseInt(result[3]),
		time: result[4],
		duration: result[5],
		progress: parseInt(result[6])/100
	};
}

const extraRegex = /volume:\s*(\d+)\%\s*repeat:\s+(\w+)\s+random:\s*(\w+)\s+single:\s*(\w+)\s+consume:\s*(\w+)/;
function parseExtraString(string) {
	const result = extraRegex.exec(string)
	return {
		volume: parseInt(result[1])/100,
		repeat: result[2] === 'on',
		random: result[3] === 'on',
		single: result[4] === 'on',
		consume: result[5] === 'on'
	};
}

function parseMpdString(string) {
	const lines = string.split('\n');
	return {
		...parseSongString(lines[0]),
		...parseStatusString(lines[1]),
		...parseExtraString(lines[2]),
	};
}

const ProviderMpd = async (extra) => {
    const mpdString = await mpd();
    return parseMpdString(mpdString);
};

export default ProviderMpd;
