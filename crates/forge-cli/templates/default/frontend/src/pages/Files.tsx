import { Heading } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

export default function Files() {
  return (
    <div className={style({ display: 'flex', flexDirection: 'column', gap: 16 })}>
      <Heading level={1}>Files</Heading>
    </div>
  );
}
