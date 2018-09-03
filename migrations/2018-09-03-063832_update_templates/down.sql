UPDATE templates 
SET data = '<html>
  <head>
    <title>Password reset on Storiqa</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      
      You received this e-mail because you have made a request to change your password. In order to do that, please follow the link below:
      
     <a href="{{reset_password_path}}/{{token}}">Reset my password on Storiqa.</a>
      <br/>
      
      Best regards,
      Storiqa Team
      <br/>
      
      Note: If you have received a password-reset email you didn&apos;t request, it&apos;s likely that someone entered your e-mail address by mistake. If you didn&apos;t initiate this request, please delete this e-mail. Your privacy and security aren&apos;t compromised by this e-mail.
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>'
WHERE
name = 'password_reset_for_user';