import fs from 'fs';
import { WebUntisSecretAuth } from 'webuntis';
import { authenticator as Authenticator } from 'otplib';

const untis = new WebUntisSecretAuth('htl-hl', process.argv[3], process.argv[4], 'melete.webuntis.com', 'custom-identity', Authenticator);

const startDate = new Date(process.argv[2]);

await untis.login();
// const startDate = new Date("2022-03-25");
const endDate = new Date();

const absentLesson = await untis.getAbsentLesson(startDate, endDate, ); // 0, only unexcused
fs.writeFileSync('absentLessons.json', JSON.stringify(absentLesson, null, 2));
