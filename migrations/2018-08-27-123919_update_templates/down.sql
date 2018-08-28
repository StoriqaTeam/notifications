UPDATE templates 
SET data = '<html>
  <head>
    <title>Verify your account on Storiqa</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      Thank you for signing up for Storiqa! In order to finish the registration process and verify your account, please confirm your e-mail by following the link below:
      
     <a href="{{verify_email_path}}/{{token}}">Verify my email on Storiqa.</a>
      <br/>

      Best regards,
      Storiqa Team
      <br/>
      
      Note: If you didn&apos;t initiate the request, please delete this e-mail. 
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>
    </p>

  </body>
</html>'
WHERE
name = 'email_verification_for_user';