import { Heading, Text } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

export default function Dashboard() {
  return (
    <>
      <Heading level={1} styles={style({ font: 'heading-xl' })} data-testid="dashboard-heading">
        Dashboard
      </Heading>
      <Text>Welcome to your dashboard.</Text>
    </>
  );
}
