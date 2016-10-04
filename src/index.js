import { initialize, order } from 'pi-spi';
import Canvas from 'canvas';
import moment from 'moment';
import mpd from './mpd';

const Register = {
	NOOP: 0x0,
	DIGIT0: 0x1,
	DIGIT1: 0x2,
	DIGIT2: 0x3,
	DIGIT3: 0x4,
	DIGIT4: 0x5,
	DIGIT5: 0x6,
	DIGIT6: 0x7,
	DIGIT7: 0x8,
	DECODEMODE: 0x9,
	INTENSITY: 0xA,
	SCANLIMIT: 0xB,
	SHUTDOWN: 0xC,
	DISPLAYTEST: 0xF
};

const spi = initialize('/dev/spidev0.0');
spi.bitOrder(order.MSB_FIRST);

function write(register, data, deviceCount, devices) {
	let deviceData = [];
	for (let device = 0; device < deviceCount; device++) {
		if (!devices || devices.indexOf(device) !== -1) {
			deviceData = [...deviceData, register, data];
		} else {
			deviceData = [...deviceData, 0, 0];
		}
	}
	spi.write(Buffer.from(deviceData), (err) => {
		if (err) {
			console.error(err);
		}
	});
}

function setup(deviceCount) {
	write(Register.SCANLIMIT, 7, deviceCount);
	write(Register.DECODEMODE, 0, deviceCount);
	write(Register.DISPLAYTEST, 0, deviceCount);
	write(Register.SHUTDOWN, 1, deviceCount);
	write(Register.INTENSITY, 1, deviceCount);
}

function clear(deviceCount) {
	for (let row = Register.DIGIT0; row <= Register.DIGIT7; row++) {
		write(row, 0, deviceCount);
	}
}

function arrayToRow(array) {
	return array.reduce((result, value, index) => result >> 1 | (value ? 128 : 0), 0);
}

function displayArray(array, width, height) {
	for (let x = 0; x < width; x++) {
		for (let y = 0; y < height; y++) {
			for (let row = 0; row < 8; row++) {
				const index = x * 8 + y * width * 64 + row * width * 8;
				const data = array.slice(index, index + 8);
				write(Register.DIGIT7 - row, arrayToRow(data), 8, [x + y * width]);
			}
		}
	}
}


function displayCanvas(canvas) {
	const width = canvas.width / 8;
	const height = canvas.height /8;
	const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
	const array = [];
	for (let i = 0; i < data.length; i += 4) {
		array.push(Boolean(data[i] === 0 && data[i + 1] === 0 && data[i + 2] === 0 && data[i + 3] === 255));
	}
	displayArray(array, width, height);
}


const canvas = new Canvas(32, 16);
const ctx = canvas.getContext('2d');

let result;
async function update() {
	result = await mpd();
}

let textOffset = 32;

function redraw() {
	ctx.clearRect(0, 0, 32, 16);
	const seconds = Math.floor(Date.now() / 1000) % 60;
	const arc = (seconds / 60) * Math.PI * 2;
	const time = moment().format('HH:mm');
	ctx.textAlign = 'center';
	ctx.fillText(time, 16, 8);
	ctx.fillRect(0, 8, 32 * result.progress, 1);
	ctx.fillRect(0, 9, 32 * result.volume, 1);
	const title = `${result.artist} - ${result.song}`;
	ctx.textAlign = 'left';
	ctx.fillText(title, textOffset, 18);
	displayCanvas(canvas);
	textOffset--;
	const { width } = ctx.measureText(title);
	if (textOffset + width < 0) {
		textOffset = 32;
	}
}

async function main() {
	ctx.addFont(new Canvas.Font('Munro', 'munro/MunroSmall.ttf'));
	ctx.font = 'Munro';
	ctx.antialias = 'none';
	ctx.textBaseline = 'bottom';
	ctx.filter = 'nearest';
	ctx.patternQuality = 'nearest';
	setup(8);
	clear(8);
	await update();
	setInterval(redraw, 50);
	setInterval(update, 500);
}

main();
