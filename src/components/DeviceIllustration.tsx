import type { DeviceCategory } from "../devices/types";

export function DeviceIllustration({ category }: { category: DeviceCategory }) {
  const common = { fill: "none", stroke: "currentColor", strokeWidth: 3, strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
  if (category === "display") return <svg viewBox="0 0 180 120" {...common}><rect x="12" y="8" width="156" height="88" rx="9"/><path d="M72 112h36M90 96v16"/><path className="screen-glow" d="M27 23h126v58H27z"/></svg>;
  if (category === "computer") return <svg viewBox="0 0 105 180" {...common}><rect x="17" y="8" width="70" height="157" rx="12"/><path d="M30 27h44M30 45h44"/><circle cx="52" cy="140" r="5"/><path d="M36 165v7m32-7v7"/></svg>;
  if (category === "keyboard") return <svg viewBox="0 0 180 100" {...common}><path d="M22 22h136l13 57H9z"/><path d="M29 39h8m10 0h8m10 0h8m10 0h8m10 0h8m10 0h8m10 0h8M26 55h10m9 0h10m9 0h10m9 0h10m9 0h10m9 0h12M43 70h94"/></svg>;
  if (category === "mouse") return <svg viewBox="0 0 100 130" {...common}><path d="M50 8c-24 0-36 18-36 47v20c0 29 13 47 36 47s36-18 36-47V55C86 26 74 8 50 8z"/><path d="M50 9v36m-12 0h24"/></svg>;
  if (category === "storage") return <svg viewBox="0 0 130 100" {...common}><rect x="10" y="23" width="83" height="55" rx="8"/><path d="M93 38h25v25H93M104 38v10m9-10v10"/><circle cx="28" cy="61" r="3"/></svg>;
  if (category === "speaker") return <svg viewBox="0 0 100 140" {...common}><rect x="15" y="8" width="70" height="122" rx="12"/><circle cx="50" cy="45" r="15"/><circle cx="50" cy="94" r="22"/></svg>;
  if (category === "microphone") return <svg viewBox="0 0 100 140" {...common}><rect x="31" y="10" width="38" height="70" rx="19"/><path d="M18 61c0 22 14 34 32 34s32-12 32-34M50 95v24m-22 10h44"/></svg>;
  if (category === "phone") return <svg viewBox="0 0 80 140" {...common}><rect x="11" y="5" width="58" height="130" rx="11"/><path d="M32 18h16M33 119h14"/></svg>;
  if (category === "midiKeyboard") return <svg viewBox="0 0 300 92" {...common}><rect className="midi-body" x="7" y="8" width="286" height="76" rx="10"/><rect x="18" y="19" width="34" height="25" rx="5"/><circle cx="29" cy="62" r="6"/><circle cx="45" cy="62" r="6"/>{[0,1,2,3,4,5,6,7,8,9,10,11].map(i => <path key={i} d={`M${62+i*18} 24v50`} />)}{[1,2,4,5,6,8,9,11].map(i => <rect key={i} x={62+i*18-6} y="24" width="12" height="28" rx="2" fill="currentColor" stroke="none"/>)}</svg>;
  if (category === "midiController") return <svg viewBox="0 0 150 105" {...common}><rect className="midi-body" x="8" y="8" width="134" height="89" rx="12"/>{[0,1,2,3,4,5,6,7].map(i => <rect key={i} x={24+(i%4)*27} y={25+Math.floor(i/4)*29} width="19" height="19" rx="4"/>)}{[0,1,2].map(i => <circle key={i} cx={34+i*30} cy="82" r="6"/>)}</svg>;
  if (category === "midiInterface") return <svg viewBox="0 0 130 88" {...common}><rect className="midi-body" x="8" y="12" width="114" height="62" rx="12"/><circle cx="42" cy="43" r="10"/><circle cx="88" cy="43" r="10"/><path d="M17 30h12m72 0h12"/></svg>;
  if (category === "usbGeneric") return <svg viewBox="0 0 130 100" {...common}><path d="M44 18h42v64H44z"/><path d="M52 18V8h26v10M58 38h14m-14 14h14m-14 14h14"/></svg>;
  if (category === "headset") return <svg viewBox="0 0 120 120" {...common}><path d="M18 67V55a42 42 0 0184 0v12"/><rect x="10" y="62" width="22" height="39" rx="9"/><rect x="88" y="62" width="22" height="39" rx="9"/><path d="M99 100c0 10-8 12-21 12"/></svg>;
  if (category === "camera") return <svg viewBox="0 0 140 100" {...common}><rect x="12" y="22" width="92" height="60" rx="10"/><circle cx="58" cy="52" r="19"/><path d="M104 40l24-12v48l-24-12"/></svg>;
  if (category === "gameController") return <svg viewBox="0 0 150 100" {...common}><path d="M45 25h60c18 0 32 38 29 53-2 11-13 12-22 2L98 65H52L38 80c-9 10-20 9-22-2-3-15 11-53 29-53z"/><path d="M40 45v18m-9-9h18"/><circle cx="105" cy="47" r="3"/><circle cx="117" cy="58" r="3"/></svg>;
  if (category === "printer") return <svg viewBox="0 0 140 110" {...common}><path d="M34 42V8h72v34M28 83H15V42h110v41h-13"/><rect x="29" y="68" width="82" height="34"/><circle cx="108" cy="55" r="3"/></svg>;
  return <svg viewBox="0 0 120 110" {...common}><rect x="15" y="20" width="90" height="70" rx="12"/><path d="M60 42v22m0 12v1"/></svg>;
}
