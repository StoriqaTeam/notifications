UPDATE templates 
SET data = '<html>
  <head>
    <title>New order {{order_slug}}</title>
  </head>
  <body>
    <p>
      Dear {{user.first_name}},
      <br/>
      Please be informed that you have a new order {{order_slug}}. 
      <br/>
      You can watch your order on <a href="{{cluster_url}}/profile/orders/{{order_slug}}">this page</a>.
      <br/>
      Best regards,
      Storiqa Team
      <br/>
      <i>This is an automatically generated e-mail – please do not reply to it.</i>

    </p>

  </body>
</html>'
WHERE
name = 'order_create_for_user';