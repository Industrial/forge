import { Button, Form, Heading, Link, TextField } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

export default function Register() {
  return (
    <>
      <Heading level={1} styles={style({ font: 'heading-xl' })}>Create an account</Heading>
      <div className={style({ display: 'flex', flexDirection: 'column', gap: 8 })}>
        <Form
          action="/api/auth/register"
          method="post"
          target="_top"
        >
          <TextField
            name="email"
            type="email"
            label="Email"
            placeholder="you@example.com"
            isRequired
          />
          <TextField
            name="password"
            type="password"
            label="Password"
            placeholder="••••••••"
            isRequired
            minLength={8}
          />
          <div
            className={style({
              display: 'flex',
              flexDirection: 'row',
              justifyContent: 'end',
            })}
          >
            <Button type="submit" variant="accent">
              Register
            </Button>
          </div>
        </Form>
        <div className={style({ textAlign: 'center', font: 'body' })}>
          <Link href="/login">Already have an account? Log in</Link>
        </div>
      </div>
    </>
  );
}
