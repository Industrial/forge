import { Heading, Text } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

export default function RolesAndPermissions() {
  return (
    <>
      <Heading level={1} styles={style({ font: 'heading-xl' })}>
        Roles and Permissions
      </Heading>
      <Text>Manage roles and permissions.</Text>
    </>
  );
}
